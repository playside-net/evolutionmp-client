use laminar::{ErrorKind, Packet, Socket, SocketEvent, DeliveryGuarantee, OrderingGuarantee};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use evolutionmp::network::{PORT, Message, PlayerData, VehicleData, STREAMING_RANGE};
use std::collections::HashMap;
use evolutionmp::hash::Hashable;
use std::time::Instant;
use cgmath::{MetricSpace, Vector3, Zero};

pub fn main() -> Result<(), ErrorKind> {
    Server::start()
}

pub struct Session {
    id: u32,
    address: SocketAddr,
    social_club: String,
    pid: u32
}

pub struct SyncedVehicle {
    streamer: Option<u32>,
    data: VehicleData,
    last_sync: Instant
}

pub struct SyncedPlayer {
    data: PlayerData,
    last_sync: Instant
}

pub struct Server {
    next_player_id: u32,
    next_vehicle_id: u32,
    sender: Box<dyn Fn(Packet) -> Result<(), Packet>>,
    connections: HashMap<SocketAddr, Session>,
    vehicles: HashMap<u32, SyncedVehicle>,
    players: HashMap<u32, SyncedPlayer>
}

impl Server {
    pub fn start() -> Result<(), ErrorKind> {
        let mut socket = Socket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT)))?;
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();

        println!("Listening on: localhost:{}", PORT);

        let _thread = std::thread::spawn(move || socket.start_polling());

        let mut server = Server {
            next_player_id: 0,
            next_vehicle_id: 0,
            sender: Box::new(move |packet| sender.send(packet).map_err(|e|e.into_inner())),
            connections: HashMap::new(),
            vehicles: HashMap::new(),
            players: HashMap::new()
        };

        loop {
            if let Ok(event) = receiver.recv() {
                match event {
                    SocketEvent::Connect(address) => server.on_connect(address),
                    SocketEvent::Packet(packet) => {
                        let address = packet.addr();
                        match bincode::deserialize(packet.payload()) {
                            Ok(message) => {
                                server.on_message(address, message)
                            },
                            Err(e) => {
                                eprintln!("Received broken message from {:?}: {:?}", address, e);
                            }
                        }
                    }
                    SocketEvent::Timeout(address) => server.on_timeout(address),
                    SocketEvent::Disconnect(address) => server.on_disconnect(address)
                }
            }
        }

        Ok(())
    }

    fn on_connect(&mut self, address: SocketAddr) {
        //println!("Incoming connection from {:?}", address); //TODO Seem to be called when timed out
    }

    fn on_disconnect(&mut self, address: SocketAddr) {
        if let Some(session) = self.connections.remove(&address) {
            println!("{} ({:?}) lost connection: Disconnected", session.social_club, address)
        }
    }

    fn on_timeout(&mut self, address: SocketAddr) {
        if let Some(session) = self.connections.remove(&address) {
            println!("{} ({:?}) timed out", session.social_club, address)
        } else {
            println!("{:?} timed out", address);
        }
    }

    fn on_message(&mut self, address: SocketAddr, message: Message) {
        match message {
            Message::Handshake { social_club, pid } => {
                let id = self.next_player_id;
                self.next_player_id += 1;
                let session = Session { id, address, social_club, pid };
                self.create_player(session, Vector3::zero());
            },
            other => {
                if let Some(session) = self.connections.get(&address) {
                    match other {
                        Message::Payload { channel, data } => {

                        },
                        Message::UpdateVehicle { id, data } => {
                            if let Some(vehicle) = self.vehicles.get_mut(&id) {
                                if vehicle.streamer.is_none() || vehicle.streamer.as_ref() == Some(&session.id) {
                                    vehicle.streamer = Some(session.id);
                                    vehicle.data = data;
                                    vehicle.last_sync = Instant::now();
                                }
                            }
                        }
                        Message::Disconnect { reason } => {},
                        _ => {}
                    }
                }
            }
        }
    }

    fn create_player(&mut self, session: Session, spawn_pos: Vector3<f32>) {
        println!("{:?} logged in as {} (pid {})", session.address, session.social_club, session.pid);
        let message = Message::CreatePlayer {
            id: session.id,
            data: PlayerData {
                position: spawn_pos,
                rotation: Vector3::zero(),
                heading: 0.0,
                model: "mp_m_freemode_01".joaat()
            }
        };
        println!("Created ped for player {} ({})", session.social_club, session.id);
        self.broadcast_reliable_sequenced(spawn_pos, STREAMING_RANGE, Some(session.address), &message, Some(0));
        if let Some(old) = self.connections.insert(session.address, session) {
            self.send_reliable_ordered(old.address, &Message::Disconnect {
                reason: String::from("You've joined from another location!")
            }, Some(0));
        }
    }

    fn send_raw(&self, address: &SocketAddr, packet: Packet) {
        match (self.sender)(packet) {
            Err(_) => {
                eprintln!("Failed to send packet to {:?}", address);
            },
            _ => {}
        }
    }

    fn broadcast_raw<P>(&self, center: Vector3<f32>, range: f32, omit: Option<SocketAddr>, packet: P)
        where P: Fn(SocketAddr) -> Packet {

        for (addr, session) in self.connections.iter() {
            if omit.is_none() || omit.as_ref() != Some(addr) {
                if let Some(player) = self.players.get(&session.id) {
                    if player.data.position.distance(center) <= range {
                        self.send_raw(addr, packet(*addr));
                    }
                }
            }
        }
    }

    fn try_serialize(&self, message: &Message) -> Option<Vec<u8>> {
        match bincode::serialize(message) {
            Ok(payload) => Some(payload),
            Err(e) => {
                eprintln!("Failed to serialize message: {:?}", e);
                None
            },
        }
    }

    pub fn send_reliable_ordered(&self, address: SocketAddr, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::reliable_ordered(address, payload, stream_id))
        }
    }

    pub fn broadcast_reliable_ordered(&self, center: Vector3<f32>, range: f32, omit: Option<SocketAddr>, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.broadcast_raw(center, range, omit, move |address| Packet::reliable_ordered(address, payload.clone(), stream_id))
        }
    }

    pub fn send_reliable_sequenced(&self, address: SocketAddr, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::reliable_sequenced(address, payload, stream_id))
        }
    }

    pub fn broadcast_reliable_sequenced(&self, center: Vector3<f32>, range: f32, omit: Option<SocketAddr>, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.broadcast_raw(center, range, omit, move |address| Packet::reliable_sequenced(address, payload.clone(), stream_id))
        }
    }

    pub fn send_reliable_unordered(&self, address: SocketAddr, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::reliable_unordered(address, payload))
        }
    }

    pub fn broadcast_reliable_unordered(&self, center: Vector3<f32>, range: f32, omit: Option<SocketAddr>, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.broadcast_raw(center, range, omit, move |address| Packet::reliable_unordered(address, payload.clone()))
        }
    }

    pub fn send_unreliable(&self, address: SocketAddr, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::unreliable(address, payload))
        }
    }

    pub fn broadcast_unreliable(&self, center: Vector3<f32>, range: f32, omit: Option<SocketAddr>, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.broadcast_raw(center, range, omit, move |address| Packet::unreliable(address, payload.clone()))
        }
    }

    pub fn send_unreliable_sequenced(&self, address: SocketAddr, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::unreliable_sequenced(address, payload, stream_id))
        }
    }

    pub fn broadcast_unreliable_sequenced(&self, center: Vector3<f32>, range: f32, omit: Option<SocketAddr>, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.broadcast_raw(center, range, omit, move |address| Packet::unreliable_sequenced(address, payload.clone(), stream_id))
        }
    }
}