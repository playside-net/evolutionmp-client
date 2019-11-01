use super::Handle;
use crate::native;
use crate::game::Vector3;
use crate::game::entity::Entity;
use crate::hash::Hashable;
use crate::game::ped::Ped;

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

    pub fn new<H>(model: H, pos: Vector3, heading: f32, is_network: bool, this_script_check: bool) -> Vehicle where H: Hashable {
        let handle = unsafe { native::vehicle::new(model, pos, heading, is_network, this_script_check) };
        Vehicle { handle }
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