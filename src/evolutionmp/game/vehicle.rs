use super::Handle;
use crate::game::Hash;
use crate::game::entity::Entity;

pub struct Vehicle {
    handle: Handle
}

impl Vehicle {
    pub fn new(model: Hash, x: f32, y: f32, z: f32, heading: f32, is_network: bool, this_script_check: bool) -> Vehicle {
        let handle = unsafe { crate::natives::vehicle::new(model, x, y, z, heading, is_network, this_script_check) };
        Vehicle { handle }
    }
}

impl Entity for Vehicle {
    fn get_handle(&self) -> Handle {
        self.handle
    }
}