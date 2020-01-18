use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;
use cgmath::Vector3;

pub fn new<H>(ty: u32, model: H, pos: Vector3<f32>, heading: f32, network: bool, this_script_check: bool) -> Handle where H: Hashable {
    invoke!(Handle, 0xD49F9B0955C367DE, ty, model.joaat(), pos, heading, network, this_script_check)
}

pub fn set_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x95E3D6257B166CF2, multiplier)
}

pub fn set_scenario_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x7A556143A1C03898, multiplier)
}

pub fn is_in_any_vehicle(ped: Handle, at_get_in: bool) -> bool {
    invoke!(bool, 0x997ABD671D25CA0B, ped, at_get_in)
}

pub fn get_in_vehicle(ped: Handle, last: bool) -> Handle {
    invoke!(Handle, 0x9A9112A0FE9A4713, ped, last)
}

pub fn get_using_vehicle(ped: Handle) -> Handle {
    invoke!(Handle, 0x6094AD011A2EA87D, ped)
}

pub fn get_entering_vehicle(ped: Handle) -> Handle {
    invoke!(Handle, 0xF92691AED837A5FC, ped)
}

pub fn put_into_vehicle(handle: Handle, vehicle: Handle, seat: i32) {
    invoke!((), 0xF75B0D629E1C063D, handle, vehicle, seat)
}

pub fn set_non_scenario_cops(enabled: bool) {
    invoke!((), 0x8A4986851C4EF6E7, enabled)
}

pub fn set_scenario_cops(enabled: bool) {
    invoke!((), 0x444CB7D7DBE6973D, enabled)
}

pub fn set_cops(enabled: bool) {
    invoke!((), 0x102E68B2024D536D, enabled)
}

pub fn set_current_weapon_visible(handle: Handle, visible: bool, deselect: bool, p3: bool, p4: bool) {
    invoke!((), 0x0725A4CCFDED9A70, handle, visible, deselect, p3, p4)
}

pub fn set_config_flag(handle: Handle, flag: u32, value: bool) {
    invoke!((), 0x1913FE4CBF41C463, handle, flag, value)
}

pub fn set_default_component_variation(handle: Handle) {
    invoke!((), 0x45EEE61580806D63, handle)
}

pub mod task {
    use crate::invoke;
    use crate::game::Handle;

    pub fn clear_immediately(handle: Handle) {
        invoke!((), 0xAAA34F8A7CB32098)
    }

    pub fn clear_secondary(handle: Handle) {
        invoke!((), 0x176CECF6F920D707, handle)
    }

    pub fn enter_vehicle(handle: Handle, vehicle: Handle, timeout: u32, seat: i32, speed: f32, flag: i32) {
        invoke!((), 0xC20E50AA46D09CA8, handle, vehicle, timeout, seat, speed, flag, 0u32)
    }

    pub fn network_move(handle: Handle, name: &str, multiplier: f32, p3: bool, dict: &str, flags: u32) {
        invoke!((), 0x2D537BA194896636, name, multiplier, p3, dict, flags)
    }

    pub fn is_network_move_active(handle: Handle) -> bool {
        invoke!(bool, 0x921CE12C489C4C41, handle)
    }

    pub fn set_network_move_signal_float(handle: Handle, name: &str, value: f32) {
        invoke!((), 0xD5BB4025AE449A4E, handle, name, value)
    }

    pub fn set_network_move_signal_bool(handle: Handle, name: &str, value: bool) {
        invoke!((), 0xB0A6CFD2C69C1088, handle, name, value)
    }

    pub fn request_network_move_state_transition(handle: Handle, name: &str) -> bool {
        invoke!(bool, 0xD01015C7316AE176, handle, name)
    }
}