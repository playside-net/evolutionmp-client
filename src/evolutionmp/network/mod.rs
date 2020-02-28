use serde_derive::{Serialize, Deserialize};
use cgmath::{Vector3, Deg, Euler, Quaternion, Rad, Matrix3};
use uuid::Uuid;
use crate::game::streaming::Model;
use crate::hash::Hash;
use crate::game::{Rgba, Rgb};
use std::collections::HashMap;

pub const PORT: u16 = 7036;

#[derive(Serialize, Deserialize)]
pub enum Message {
    Handshake {
        socialclub: String,
        pid: u32
    },
    LoggedIn {
        id: Uuid,
    },
    Chat {
        message: String
    },
    Disconnect {
        reason: String
    },
    CreateVehicle {
        id: Uuid,
        model: Hash,
        data: VehicleData
    },
    UpdateVehicle {
        id: Uuid,
        streamer: Uuid,
        data: VehicleData
    },
    CreatePlayer {
        id: Uuid,
        model: Hash,
        data: PlayerData
    },
    UpdatePlayer {
        id: Uuid,
        data: PlayerData
    }
}

#[derive(Serialize, Deserialize)]
pub struct VehicleData {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub heading: f32,
    pub forward_speed: f32,
    pub engine_on: bool,
    pub engine_health: f32,
    pub gears: i32,
    pub rpm: f32,
    pub clutch: f32,
    pub turbo: f32,
    pub throttle: f32,
    pub acceleration: f32,
    pub brake: f32,
    pub wheel_speed: f32,
    pub steering_angle: Rad<f32>,
    pub steering_scale: f32,
    //pub forward_wheel_angle: f32,
    pub colors: [VehicleColor; 2],
    pub mods: HashMap<u8, u8>,
    pub extras: u16,
    pub plate_number: String,
    pub plate_style: u32,
    pub doors_lock_state: u32
}

#[derive(Serialize, Deserialize)]
pub enum VehicleColor {
    Standard {
        color: u8,
        ty: u8
    },
    Custom {
        color: Rgb
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlayerData {

}