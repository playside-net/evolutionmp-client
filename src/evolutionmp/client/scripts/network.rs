use std::net::SocketAddr;
use std::sync::Mutex;

use bimap::BiMap;
use laminar::{EventReceiver, Packet, PacketSender, Socket, Config};

use crate::events::ScriptEvent;
use crate::game::ped::Ped;
use crate::game::ui::FrontendButtons;
use crate::game::vehicle::Vehicle;
use crate::network::Message;
use crate::runtime::Script;
use std::time::Duration;

lazy_static! {
    static ref PLAYERS: Mutex<BiMap<u32, Ped>> = Mutex::new(BiMap::new());
    static ref VEHICLES: Mutex<BiMap<u32, Vehicle>> = Mutex::new(BiMap::new());
}

pub fn get_remote_player(remote_id: u32) -> Option<Ped> {
    let players = PLAYERS.lock().unwrap();
    players.get_by_left(&remote_id).cloned()
}

pub fn get_player_remote_id(player: &Ped) -> Option<u32> {
    let players = PLAYERS.lock().unwrap();
    players.get_by_right(player).cloned()
}

pub fn get_remote_vehicle(remote_id: u32) -> Option<Vehicle> {
    let vehicles = VEHICLES.lock().unwrap();
    vehicles.get_by_left(&remote_id).cloned()
}

pub fn get_vehicle_remote_id(vehicle: &Vehicle) -> Option<u32> {
    let vehicles = VEHICLES.lock().unwrap();
    vehicles.get_by_right(vehicle).cloned()
}

pub struct ScriptNetwork {
    sender: Option<PacketSender>,
    receiver: Option<EventReceiver>,
    server: SocketAddr,
}

impl ScriptNetwork {
    pub fn new(server: SocketAddr) -> ScriptNetwork {
        ScriptNetwork {
            sender: None,
            receiver: None,
            server,
        }
    }

    fn serialize(message: &Message) -> Option<Vec<u8>> {
        match bincode::serialize(message) {
            Ok(data) => Some(data),
            Err(e) => {
                error!("Error serializing message `{:?}`: {:?}", message, e);
                None
            }
        }
    }

    fn send_reliable_ordered(&self, message: &Message, stream_id: Option<u8>) -> bool {
        if let Some(sender) = self.sender.as_ref() {
            if let Some(payload) = Self::serialize(message) {
                if let Ok(()) = sender.send(Packet::reliable_ordered(self.server, payload, stream_id)) {
                    return true;
                }
            }
        }
        false
    }

    fn connection_failed(&self, reason: &str) -> bool {
        match crate::game::ui::warn(
            "Connection error",
            "Server connection failed",
            reason,
            FrontendButtons::RetryCancel,
            true,
        ) {
            FrontendButtons::Retry => {
                info!("Trying to reconnect...");
                true
            }
            _ => {
                info!("Exiting game");
                std::process::exit(-1);
            }
        }
    }
}

impl Script for ScriptNetwork {
    fn frame(&mut self) {
        if let Some(receiver) = self.receiver.as_mut() {
            let local_player = Ped::local();
            if let Some(local_vehicle) = local_player.get_in_vehicle(false) {}
            if let Ok(event) = receiver.try_recv() {
                info!("Received net event: {:?}", event);
                /*if self.connection_failed("Timed out") {
                    self.receiver = None;
                    return;
                }*/
            }
        } else if crate::game::is_loaded() {
            loop {
                match Socket::bind_any_with_config(Config {
                    idle_connection_timeout: Duration::from_secs(15),
                    heartbeat_interval: Some(Duration::from_secs(10)),
                    .. Default::default()
                }) {
                    Ok(mut socket) => {
                        let sender = socket.get_packet_sender();
                        let receiver = socket.get_event_receiver();
                        self.sender = Some(sender);
                        self.receiver = Some(receiver);
                        let social_club = crate::game::player::get_social_club().to_string();
                        if !self.send_reliable_ordered(&Message::Handshake {
                            social_club,
                            pid: std::process::id(),
                        }, Some(0)) {
                            error!("Handshake failed");
                            /*if self.connection_failed("Handshake failed") {
                                continue;
                            }*/
                        }
                        let _thread = std::thread::spawn(move || socket.start_polling());
                        break;
                    }
                    Err(e) => {
                        error!("Server connection failed: {}", e);
                        /*if self.connection_failed(&format!("Socket error: {}", e)) {
                            continue;
                        }*/
                    }
                }
            }
        }
    }

    fn event(&mut self, _event: ScriptEvent) {}
}