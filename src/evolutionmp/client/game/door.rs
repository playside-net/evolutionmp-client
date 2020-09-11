use crate::{invoke, invoke_option};
use crate::hash::{Hash, Hashable};
use cgmath::Vector3;

pub struct Door {
    hash: Hash
}

impl Door {
    pub fn new<H>(hash: H) -> Door where H: Hashable {
        Door {
            hash: hash.joaat()
        }
    }

    pub fn get_handle_at(&self, position: Vector3<f32>) -> Option<u64> {
        let mut handle = 0;
        invoke_option!(handle, 0x589F80B325CC82C5, position, self.hash, &mut handle)
    }

    pub fn set_locked(&self, position: Vector3<f32>, locked: bool, rotation_speed: Vector3<f32>) {
        invoke!((), 0x9B12F9A24FABEDB0, self.hash, position, locked, rotation_speed)
    }

    pub fn is_closed(&self) -> bool {
        invoke!(bool, 0xC531EE8A1145A149, self.hash)
    }

    pub fn is_registered(&self) -> bool {
        invoke!(bool, 0xC153C43EA202C8C1, self.hash)
    }

    pub fn unregister(&self) {
        invoke!((), 0x464D8E1427156FE4, self.hash)
    }

    pub fn register<M>(&self, model: M, position: Vector3<f32>, unknown: Vector3<bool>) where M: Hashable {
        invoke!((), 0x6F8838D03D1DC226, self.hash, model.joaat(), position, unknown)
    }

    pub fn get_closest_state_at(&self, position: Vector3<f32>) -> (bool, f32) {
        let mut locked = false;
        let mut heading = 0.0;
        invoke!((), 0xEDC1A5B84AEF33FF, self.hash, position, &mut locked, &mut heading);
        (locked, heading)
    }

    pub fn set_closest_state_at(&self, position: Vector3<f32>, locked: bool, heading: f32) {
        invoke!((), 0xF82D8F1926A02C3D, self.hash, position, locked, heading)
    }

    pub fn set_spring_removed(&self, unknown: Vector3<bool>) {
        invoke!((), 0xC485E07E4F0B7958, self.hash, unknown)
    }

    pub fn set_open_ratio(&self, ratio: f32) {
        invoke!((), 0xB6E6FBA95C7324AC, self.hash, ratio, false, true)
    }

    pub fn get_open_ratio(&self) -> f32 {
        invoke!(f32, 0x65499865FCA6E5EC, self.hash)
    }

    pub fn set_hold_open(&self, hold_open: bool) {
        invoke!((), 0xD9B71952F78A2640, self.hash, hold_open)
    }

    pub fn set_acceleration_limit(&self, limit: u32) {
        invoke!((), 0x6BAB9442830C7F53, self.hash, limit, false, true)
    }

    pub fn set_automatic_rate(&self, rate: f32) {
        invoke!((), 0x03C27E13B42A0E82, self.hash, rate, false, true)
    }

    pub fn set_automatic_distance(&self, distance: f32) {
        invoke!((), 0x9BA001CB45CBF627, self.hash, distance, false, true)
    }

    pub fn is_physics_loaded(&self) -> bool {
        invoke!(bool, 0xDF97CDD4FC08FD34, self.hash)
    }

    pub fn get_state(&self) -> u32 {
        invoke!(u32, 0x160AA1B32F6139B8, self.hash)
    }

    pub fn get_pending_state(&self) -> u32 {
        invoke!(u32, 0x4BC2854478F3A749, self.hash)
    }
}