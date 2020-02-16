use crate::runtime::{Script, ScriptEnv, Runtime};
use crate::network::{Message, PORT, VehicleData, VehicleColor};
use crate::events::ScriptEvent;
use crate::game::{self, Handle};
use crate::game::vehicle::{Vehicle, VehicleClass};
use crate::native::pool::Handleable;
use crate::game::entity::Entity;
use crate::game::ped::Ped;
use crate::hash::Hash;
use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4, IpAddr};
use std::time::Duration;
use std::collections::{VecDeque, HashMap};
use std::thread::JoinHandle;
use uuid::Uuid;
use laminar::{Packet, Socket, ErrorKind, DeliveryGuarantee, OrderingGuarantee, SocketEvent};
use cgmath::Vector3;
use crate::game::player::Player;
use crate::game::streaming::Model;

const REMOTE_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);//Ipv4Addr::new(116, 202, 4, 42);

fn get_remote_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(REMOTE_IP, PORT))
}

pub fn init(runtime: &mut Runtime) {
    match Socket::bind_any() {
        Ok(socket) => {
            runtime.register_script("network", ScriptNetwork::new(socket));
        },
        Err(e) => {
            crate::error!("Local server binding failed: {:?}", e)
        },
    }
}

pub struct ScriptNetwork {
    receiver: Box<dyn FnMut() -> Option<SocketEvent>>,
    sender: Box<dyn Fn(Packet) -> Result<(), Packet>>,
    thread: JoinHandle<()>,
    session_id: Option<Uuid>,
    players: HashMap<Uuid, SyncedPlayer>,
    vehicles: HashMap<Uuid, SyncedVehicle>
}

impl Script for ScriptNetwork {
    fn prepare(&mut self, mut env: ScriptEnv) {
        self.send_reliable_sequenced(&Message::Handshake {
            socialclub: super::game::player::get_social_club().to_owned(),
            pid: std::process::id()
        }, Some(0))
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        while let Some(event) = (self.receiver)() {
            match event {
                SocketEvent::Packet(packet) => {
                    let address = packet.addr();
                    if address.ip() == IpAddr::V4(REMOTE_IP) {
                        match bincode::deserialize(packet.payload()) {
                            Ok(message) => {
                                self.on_message(&mut env, message)
                            },
                            Err(e) => {
                                eprintln!("Received broken message from server: {:?}", e);
                            }
                        }
                    }
                }
                SocketEvent::Timeout(address) => {
                    if address.ip() == IpAddr::V4(REMOTE_IP) {
                        self.on_timeout(&mut env)
                    }
                },
                _ => {}
            }
        }

        self.vehicles.retain(|id, veh| veh.handle.exists());

        if let Some(session_id) = self.session_id {
            self.update_vehicles(&session_id);
            self.update_players(&session_id);
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        match event {
            ScriptEvent::ConsoleInput(message) => {
                self.send_reliable_unordered(&Message::Chat {
                    message: message.clone()
                })
            },
            _ => {}
        }
        false
    }
}

impl ScriptNetwork {
    pub fn new(mut socket: Socket) -> ScriptNetwork {
        let receiver = socket.get_event_receiver();
        let sender = socket.get_packet_sender();
        ScriptNetwork {
            receiver: Box::new(move || receiver.try_recv().ok()),
            sender: Box::new(move |packet| sender.send(packet).map_err(|e|e.into_inner())),
            thread: std::thread::spawn(move || socket.start_polling()),
            session_id: None,
            players: HashMap::new(),
            vehicles: HashMap::new()
        }
    }

    fn send_raw(&self, packet: Packet) {
        match (self.sender)(packet) {
            Err(_) => {
                eprintln!("Failed to send packet to server");
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

    pub fn send_reliable_ordered(&self, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(Packet::reliable_ordered(get_remote_address(), payload, stream_id))
        }
    }

    pub fn send_reliable_sequenced(&self, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(Packet::reliable_sequenced(get_remote_address(), payload, stream_id))
        }
    }

    pub fn send_reliable_unordered(&self, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(Packet::reliable_unordered(get_remote_address(), payload))
        }
    }

    pub fn send_unreliable(&self, message: &Message) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(Packet::unreliable(get_remote_address(), payload))
        }
    }

    pub fn send_unreliable_sequenced(&self, message: &Message, stream_id: Option<u8>) {
        if let Some(payload) = self.try_serialize(message) {
            self.send_raw(Packet::unreliable_sequenced(get_remote_address(), payload, stream_id))
        }
    }

    fn on_connect(&mut self) {}

    fn on_timeout(&mut self, env: &mut ScriptEnv) {
        println!("Timed out");
    }

    fn on_message(&mut self, env: &mut ScriptEnv, message: Message) {
        match message {
            Message::Disconnect { reason } => {},
            Message::LoggedIn { id } => {
                self.session_id = Some(id);
            }
            Message::Chat { message } => env.log(message),
            _ => {}
        }
    }

    fn update_vehicles(&mut self, session_id: &Uuid) {
        if let Some((id, veh)) = self.vehicles.iter().find(|(id, veh)| {
            veh.streamer.as_ref() == Some(session_id)
        }) {
            self.sync_driven_vehicle(*id, &veh.handle, *session_id);
        }
        for (id, veh) in self.vehicles.iter_mut() {
            veh.update(session_id);
        }
    }

    fn sync_driven_vehicle(&self, id: Uuid, veh: &Vehicle, session_id: Uuid) {
        let position = veh.get_position();
        let rotation = veh.get_rotation(2);
        let velocity = veh.get_velocity();
        let engine_on = veh.is_engine_on();
        let gear = veh.get_gear();
        let rpm = veh.get_current_rpm();
        //TODO: Clutch, turbo, brake
        let acceleration = veh.get_acceleration();
        let steering_angle = veh.get_steering_angle();
        let steering_scale = veh.get_steering_scale();
        let wheel_speed = veh.get_wheel_speed();
        let colors = veh.get_colors();
        let data = VehicleData {
            position,
            rotation,
            velocity,
            heading: 0.0,
            forward_speed: 0.0,
            engine_on,
            //engine_health: 0.0,
            gear: 0,
            rpm,
            //clutch: 0.0,
            //turbo: 0.0,
            acceleration,
            //brake: 0.0,
            wheel_speed,
            steering_angle,
            steering_scale,
            //forward_wheel_angle: 0.0,
            colors: [
                VehicleColor::Standard { color: colors.primary as u8, ty: 0 },
                VehicleColor::Standard { color: colors.secondary as u8, ty: 0 }
            ],
            mods: HashMap::new(),
            extras: 0b0000000000000000,
            plate_number: String::new(),
            plate_style: 0,
            doors_lock_state: 0
        };
        self.send_unreliable_sequenced(&Message::UpdateVehicle {
            id,
            streamer: session_id,
            data
        }, Some(0));
    }

    fn update_players(&mut self, session_id: &Uuid) {

    }
}

struct SyncedVehicle {
    id: Uuid,
    streamer: Option<Uuid>,
    handle: Vehicle
}

struct SyncedPlayer {
    id: Uuid,
    handle: Ped
}

impl SyncedVehicle {
    fn new(env: &mut ScriptEnv, id: Uuid, model: Hash, data: VehicleData) -> Option<SyncedVehicle> {
        let handle = Vehicle::new(env, Model::from(model), data.position, data.heading, false, false)?;
        handle.set_position_freezed(true);
        game::streaming::request_collision_at(data.position);
        handle.set_position_no_offset(data.position, Vector3::new(false, false, false));
        handle.set_load_collision(true);
        handle.set_collision(true, false);
        handle.set_rotation(data.rotation, 2);
        handle.set_taxi_lights(true);
        if !handle.get_class().has_custom_horns() {
            handle.set_mod(48, 0, false);
            handle.set_livery(0);
        }
        handle.set_position_freezed(false);
        handle.set_dynamic(true);

        Some(SyncedVehicle { id, handle, streamer: None })
    }

    fn delete(&mut self) {
        self.handle.delete();
    }

    fn update(&mut self, session_id: &Uuid) {

    }

    fn sync(&mut self, data: VehicleData) {

    }
}