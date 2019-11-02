use super::Handle;
use crate::native;
use crate::game::Vector3;
use crate::game::entity::Entity;
use crate::hash::Hashable;
use crate::game;
use crate::game::ped::Ped;
use crate::game::streaming::Model;
use winapi::_core::time::Duration;
use crate::runtime::ScriptEnv;

pub struct Vehicle {
    handle: Handle
}

impl Vehicle {
    pub unsafe fn from_handle(handle: Handle) -> Option<Vehicle> {
        if handle == 0 {
            None
        } else {
            Some(Vehicle { handle })
        }
    }

    pub fn new<H>(env: &mut ScriptEnv, model: H, pos: Vector3<f32>, heading: f32, is_network: bool, this_script_check: bool) -> Option<Vehicle> where H: Hashable {
        let model = Model::new(model);
        if model.is_in_cd_image() && model.is_valid() && model.is_vehicle() {
            unsafe {
                model.request_and_wait(env);
                let handle = native::vehicle::new(model, pos, heading, is_network, this_script_check);
                model.mark_unused();
                if handle != 0 {
                    return Some(Vehicle { handle });
                }
            }
        }
        None
    }

    pub fn get_colors(&self) -> VehicleColors {
        let mut primary = 0;
        let mut secondary = 0;
        unsafe { native::vehicle::get_colors(self.handle, &mut primary, &mut secondary) }
        VehicleColors { primary, secondary }
    }

    pub fn set_colors(&self, primary: u32, secondary: u32) {
        unsafe { native::vehicle::set_colors(self.handle, primary, secondary) }
    }

    pub fn repair(&self) {
        unsafe { native::vehicle::repair(self.handle) }
    }

    pub fn repair_deformation(&self) {
        unsafe { native::vehicle::repair_deformation(self.handle) }
    }

    pub fn place_on_ground(&self) {
        unsafe { native::vehicle::place_on_ground(self.handle) }
    }
}

pub struct VehicleColors {
    pub primary: u32,
    pub secondary: u32
}

impl Entity for Vehicle {
    fn get_handle(&self) -> Handle {
        self.handle
    }
}