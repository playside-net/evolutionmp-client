use laminar::{ErrorKind, Packet, Socket, SocketEvent, DeliveryGuarantee, OrderingGuarantee};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use evolutionmp::network::{PORT, Message, PlayerData};
use std::collections::HashMap;
use uuid::Uuid;
use evolutionmp::hash::Hashable;

pub fn main() -> Result<(), ErrorKind> {
    Server::start()
}

pub struct Session {
    address: SocketAddr,
    socialclub: String,
    pid: u32
}

pub struct Server {
    sender: Box<dyn Fn(Packet) -> Result<(), Packet>>,
    connections: HashMap<SocketAddr, Session>
}

impl Server {
    pub fn start() -> Result<(), ErrorKind> {
        let mut socket = Socket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT)))?;
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();

        println!("Listening on: localhost:{}", PORT);

        let _thread = std::thread::spawn(move || socket.start_polling());

        let mut server = Server {
            sender: Box::new(move |packet| sender.send(packet).map_err(|e|e.into_inner())),
            connections: HashMap::new()
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
                }
            }
        }

        Ok(())
    }

    fn on_connect(&mut self, address: SocketAddr) {
        //println!("Incoming connection from {:?}", address); //TODO Seem to be called when timed out
    }

    fn on_timeout(&mut self, address: SocketAddr) {
        if let Some(session) = self.connections.remove(&address) {
            println!("{} ({:?}) timed out", session.socialclub, address)
        } else {
            println!("{:?} timed out", address);
        }
    }

    fn on_message(&mut self, address: SocketAddr, message: Message) {
        match message {
            Message::Handshake { socialclub, pid } => {
                let session = Session { address, socialclub, pid };
                self.create_player(session);
            },
            other => {
                if let Some(session) = self.connections.get(&address) {
                    match other {
                        Message::Chat { message } => {
                            println!("[{}]: {}", session.socialclub, message);
                        },
                        Message::Disconnect { reason } => {},
                        _ => {}
                    }
                }
            }
        }
    }

    fn on_chat(&mut self, address: SocketAddr, message: String) {

    }

    fn create_player(&mut self, session: Session) {
        println!("{:?} logged in as {} (pid {})", session.address, session.socialclub, session.pid);
        let id = Uuid::new_v4();
        let message = Message::CreatePlayer {
            id,
            model: "mp_m_freemode_01".joaat(),
            data: PlayerData {}
        };
        println!("Created ped for player {} ({})", session.socialclub, id);
        for conn in self.connections.values() {
            if conn.address != session.address {
                self.send_reliable_sequenced(conn.address, &message, Some(0));
            }
        }
        if let Some(old) = self.connections.insert(session.address, session) {

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

    pub fn send_reliable_sequenced(&self, address: SocketAddr, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::reliable_sequenced(address, payload, stream_id))
        }
    }

    pub fn send_reliable_unordered(&self, address: SocketAddr, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::reliable_unordered(address, payload))
        }
    }

    pub fn send_unreliable(&self, address: SocketAddr, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::unreliable(address, payload))
        }
    }

    pub fn send_unreliable_sequenced(&self, address: SocketAddr, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(&address, Packet::unreliable_sequenced(address, payload, stream_id))
        }
    }
}