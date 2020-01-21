use super::Handle;
use crate::invoke;
use crate::game::entity::Entity;
use crate::hash::Hashable;
use crate::game;
use crate::game::ped::Ped;
use crate::game::streaming::Model;
use crate::runtime::ScriptEnv;
use crate::native::vehicle::GEARS_OFFSET;
use crate::native::pool::Handleable;
use std::time::Duration;
use std::sync::atomic::Ordering;
use cgmath::Vector3;

#[derive(Debug)]
pub struct Vehicle {
    handle: Handle
}

pub fn set_parked_count(count: i32) {
    invoke!((), 0xCAA15F13EBD417FF, count)
}

pub fn set_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x245A6883D966D537, multiplier)
}

pub fn set_random_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0xB3B3359379FE77D3, multiplier)
}

pub fn set_parked_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0xEAE6DCC7EEE3DB1D, multiplier)
}

pub fn set_garbage_trucks(enabled: bool) {
    invoke!((), 0x2AFD795EEAC8D30D, enabled)
}

pub fn set_random_boats(enabled: bool) {
    invoke!((), 0x84436EC293B1415F, enabled)
}

pub fn set_random_trains(enabled: bool) {
    invoke!((), 0x80D9F74197EA47D9, enabled)
}

pub fn set_far_draw(enabled: bool) {
    invoke!((), 0x26324F33423F3CC3, enabled)
}

pub fn set_distant_visible(visible: bool) {
    invoke!((), 0xF796359A959DF65D, visible)
}

pub fn set_distant_lights_visible(visible: bool) {
    invoke!((), 0xC9F98AC1884E73A2, !visible)
}

pub fn delete_all_trains() {
    invoke!((), 0x736A718577F39C7D)
}

pub fn set_low_priority_generators_active(active: bool) {
    invoke!((), 0x608207E7A8FB787C, active)
}

pub fn remove_vehicles_from_generators_in_area(start: Vector3<f32>, end: Vector3<f32>, unknown: bool) {
    invoke!((), 0x46A1E1A299EC4BBA, start, end, unknown)
}

impl Vehicle {
    pub fn new<H>(env: &mut ScriptEnv, model: H, pos: Vector3<f32>, heading: f32, is_network: bool, this_script_check: bool) -> Option<Vehicle> where H: Hashable {
        let model = Model::new(model);
        if model.is_in_cd_image() && model.is_valid() && model.is_vehicle() {
            env.wait_for_resource(&model);
            let result = invoke!(Option<Vehicle>, 0xAF35D0D2583051B0, model.joaat(), pos, heading, is_network, this_script_check);
            model.mark_unused();
            result
        } else {
            None
        }
    }

    pub fn is_radio_loud(&self) -> bool {
        invoke!(bool, 0x032A116663A4D5AC, self.handle)
    }

    pub fn get_colors(&self) -> VehicleColors {
        let mut primary = 0;
        let mut secondary = 0;
        invoke!((), 0xA19435F193E081AC, self.handle, &mut primary, &mut secondary);
        VehicleColors { primary, secondary }
    }

    pub fn set_colors(&self, primary: u32, secondary: u32) {
        invoke!((), 0x4F1D4BE3A7F24601, self.handle, primary, secondary)
    }

    pub fn get_gears_offset(&self) -> i32 {
        unsafe {
            self.get_address().offset(GEARS_OFFSET.load(Ordering::SeqCst) as isize).cast::<i32>().read()
        }
    }

    pub fn get_passenger(&self, seat: i32) -> Option<Ped> {
        invoke!(Option<Ped>, 0xBB40DD2270B65366, self.handle)
    }

    pub fn get_max_passengers(&self) -> u32 {
        invoke!(u32, 0xA7C4F2C6E744A550, self.handle)
    }

    pub fn is_seat_free(&self, seat: i32) -> bool {
        invoke!(bool, 0x22AC59A870E6A669, self.handle, seat)
    }

    pub fn repair(&self) {
        invoke!((), 0x115722B1B9C14C1C, self.handle)
    }

    pub fn repair_deformation(&self) {
        invoke!((), 0x953DA1E1B12C0491, self.handle)
    }

    pub fn place_on_ground(&self) {
        invoke!((), 0x49733E92263139D1, self.handle)
    }

    pub fn start_horn<H>(&self, duration: u32, hash: H, forever: bool) where H: Hashable {
        invoke!((), 0x9C8C6504B5B63D2C, self.handle, duration, hash.joaat(), forever)
    }

    pub fn get_waypoint_progress(&self) -> f32 {
        invoke!(f32, 0x9824CFF8FC66E159, self.handle)
    }

    pub fn get_waypoint_target_point(&self) -> Handle {
        invoke!(Handle, 0x416B62AC8B9E5BBD, self.handle)
    }
}

pub struct VehicleColors {
    pub primary: u32,
    pub secondary: u32
}

impl Entity for Vehicle {
    fn delete(&mut self) {
        self.set_persistent(false);
        invoke!((), 0xEA386986E786A54F, &mut self.handle)
    }
}

impl Handleable for Vehicle {
    fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }

    fn get_handle(&self) -> Handle {
        self.handle
    }
}