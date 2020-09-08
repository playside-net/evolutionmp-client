use serde_derive::{Deserialize, Serialize};
use cgmath::Vector3;
use crate::hash::Hash;

pub static PORT: u16 = 4242;
pub static STREAMING_RANGE: f32 = 512.0;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Handshake {
        social_club: String,
        pid: u32
    },
    Payload {
        channel: String,
        data: Vec<u8>
    },
    UpdateVehicle {
        id: u32,
        data: VehicleData
    },
    CreatePlayer {
        id: u32,
        data: PlayerData
    },
    Disconnect {
        reason: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerData {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub heading: f32,
    pub model: Hash
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VehicleData {

}