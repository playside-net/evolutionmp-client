use super::Handle;
use crate::native;
use crate::game::entity::Entity;
use crate::hash::Hashable;
use crate::game;
use crate::game::ped::Ped;
use crate::game::streaming::Model;
use crate::runtime::ScriptEnv;
use crate::native::vehicle::GEARS_OFFSET;
use crate::native::pool::FromHandle;
use std::time::Duration;
use std::sync::atomic::Ordering;
use cgmath::Vector3;

pub struct Vehicle {
    handle: Handle
}

impl Vehicle {
    pub fn new<H>(env: &mut ScriptEnv, model: H, pos: Vector3<f32>, heading: f32, is_network: bool, this_script_check: bool) -> Option<Vehicle> where H: Hashable {
        let model = Model::new(model);
        if model.is_in_cd_image() && model.is_valid() && model.is_vehicle() {
            env.wait_for_resource(&model);
            let handle = native::vehicle::new(model, pos, heading, is_network, this_script_check);
            model.mark_unused();
            if handle != 0 {
                return Some(Vehicle { handle });
            }
        }
        None
    }

    pub fn get_colors(&self) -> VehicleColors {
        let mut primary = 0;
        let mut secondary = 0;
        native::vehicle::get_colors(self.handle, &mut primary, &mut secondary);
        VehicleColors { primary, secondary }
    }

    pub fn set_colors(&self, primary: u32, secondary: u32) {
        native::vehicle::set_colors(self.handle, primary, secondary)
    }

    pub fn get_gears_offset(&self) -> i32 {
        unsafe {
            self.get_address().offset(GEARS_OFFSET.load(Ordering::SeqCst) as isize).cast::<i32>().read()
        }
    }

    pub fn repair(&self) {
        native::vehicle::repair(self.handle)
    }

    pub fn repair_deformation(&self) {
        native::vehicle::repair_deformation(self.handle)
    }

    pub fn place_on_ground(&self) {
        native::vehicle::place_on_ground(self.handle)
    }

    pub fn start_horn<H>(&self, duration: u32, hash: H, forever: bool) where H: Hashable {
        native::vehicle::start_horn(self.handle, duration, hash, forever)
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

    fn delete(&mut self) {
        self.set_persistent(false);
        native::entity::delete(&mut self.handle)
    }
}

impl FromHandle for Vehicle {
    fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }
}