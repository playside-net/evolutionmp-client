use crate::invoke;
use crate::game::Handle;

pub unsafe fn set_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x95E3D6257B166CF2, multiplier)
}

pub unsafe fn set_scenario_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x7A556143A1C03898, multiplier)
}

pub unsafe fn is_in_any_vehicle(ped: Handle, at_get_in: bool) -> bool {
    invoke!(bool, 0x997ABD671D25CA0B, ped, at_get_in)
}

pub unsafe fn get_in_vehicle(ped: Handle, last: bool) -> Handle {
    invoke!(Handle, 0x9A9112A0FE9A4713, ped, last)
}

pub unsafe fn get_using_vehicle(ped: Handle) -> Handle {
    invoke!(Handle, 0x6094AD011A2EA87D, ped)
}

pub unsafe fn get_entering_vehicle(ped: Handle) -> Handle {
    invoke!(Handle, 0xF92691AED837A5FC, ped)
}

pub unsafe fn clear_tasks_immediately(handle: Handle) {
    invoke!((), 0xAAA34F8A7CB32098)
}

pub unsafe fn put_into_vehicle(handle: Handle, vehicle: Handle, seat: i32) {
    invoke!((), 0xF75B0D629E1C063D, handle, vehicle, seat)
}

pub unsafe fn set_non_scenario_cops(enabled: bool) {
    invoke!((), 0x8A4986851C4EF6E7, enabled)
}

pub unsafe fn set_scenario_cops(enabled: bool) {
    invoke!((), 0x444CB7D7DBE6973D, enabled)
}

pub unsafe fn set_cops(enabled: bool) {
    invoke!((), 0x102E68B2024D536D, enabled)
}