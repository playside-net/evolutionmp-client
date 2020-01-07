use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;
use cgmath::Vector3;
use crate::pattern::MemoryRegion;

pub(crate) static mut GEARS_OFFSET: i32 = 0;
pub(crate) static mut HIGH_GEAR_OFFSET: i32 = 0;
pub(crate) static mut FUEL_LEVEL_OFFSET: i32 = 0;
pub(crate) static mut WHEEL_SPEED_OFFSET: i32 = 0;
pub(crate) static mut CURRENT_RPM_OFFSET: i32 = 0;
pub(crate) static mut ACCELERATION_OFFSET: i32 = 0;
pub(crate) static mut STEERING_SCALE_OFFSET: i32 = 0;
pub(crate) static mut STEERING_ANGLE_OFFSET: i32 = 0;

pub unsafe fn init(mem: &MemoryRegion) {
    let address = mem.find("48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24")
        .next().expect("gear offset")
        .add(3);
    GEARS_OFFSET = *address.get::<i32>() + 2;
    HIGH_GEAR_OFFSET = *address.get::<i32>() + 6;

    let address = mem.find("74 ? 0F 57 C9 0F 2F 8B ? ? ? ? 73 ? F3 0F 10 83 ? ? ? ?")
        .next().expect("fuel level offset")
        .add(8);
    FUEL_LEVEL_OFFSET = *address.get::<i32>();

    let address = mem.find("F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?")
        .next().expect("wheel speed offset")
        .add(4);

    WHEEL_SPEED_OFFSET = *address.get::<i32>();

    let address = mem.find("76 03 0F 28 F0 F3 44 0F 10 93")
        .next().expect("rpm offset")
        .add(10);
    CURRENT_RPM_OFFSET = *address.get::<i32>();
    ACCELERATION_OFFSET = *address.get::<i32>() + 16;

    let address = mem.find("74 0A F3 0F 11 B3 ? ? ? ? EB 25")
        .next().expect("steering offset")
        .add(6);
    STEERING_SCALE_OFFSET = *address.get::<i32>();
    STEERING_ANGLE_OFFSET = *address.get::<i32>() + 8;
}

pub unsafe fn new<H>(model: H, pos: Vector3<f32>, heading: f32, is_network: bool, this_script_check: bool) -> Handle where H: Hashable {
    invoke!(Handle, 0xAF35D0D2583051B0, model.joaat(), pos, heading, is_network, this_script_check)
}

pub unsafe fn set_parked_count(count: i32) {
    invoke!((), 0xCAA15F13EBD417FF, count)
}

pub unsafe fn set_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x245A6883D966D537, multiplier)
}

pub unsafe fn set_random_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0xB3B3359379FE77D3, multiplier)
}

pub unsafe fn set_parked_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0xEAE6DCC7EEE3DB1D, multiplier)
}

pub unsafe fn is_radio_loud(vehicle: Handle) -> bool {
    invoke!(bool, 0x032A116663A4D5AC, vehicle)
}

pub unsafe fn get_colors(vehicle: Handle, primary: &mut u32, secondary: &mut u32) {
    invoke!((), 0xA19435F193E081AC, vehicle, primary, secondary)
}

pub unsafe fn set_colors(vehicle: Handle, primary: u32, secondary: u32) {
    invoke!((), 0x4F1D4BE3A7F24601, primary, secondary)
}

pub unsafe fn repair(vehicle: Handle) {
    invoke!((), 0x115722B1B9C14C1C, vehicle)
}

pub unsafe fn repair_deformation(vehicle: Handle) {
    invoke!((), 0x953DA1E1B12C0491, vehicle)
}

pub unsafe fn place_on_ground(vehicle: Handle) {
    invoke!((), 0x49733E92263139D1, vehicle)
}

pub unsafe fn set_garbage_trucks(enabled: bool) {
    invoke!((), 0x2AFD795EEAC8D30D, enabled)
}

pub unsafe fn set_random_boats(enabled: bool) {
    invoke!((), 0x84436EC293B1415F, enabled)
}

pub unsafe fn set_random_trains(enabled: bool) {
    invoke!((), 0x80D9F74197EA47D9, enabled)
}

pub unsafe fn set_far_draw(enabled: bool) {
    invoke!((), 0x26324F33423F3CC3, enabled)
}

pub unsafe fn set_distant_visible(visible: bool) {
    invoke!((), 0xF796359A959DF65D, visible)
}

pub unsafe fn set_distant_lights_visible(visible: bool) {
    invoke!((), 0xC9F98AC1884E73A2, !visible)
}

pub unsafe fn delete_all_trains() {
    invoke!((), 0x736A718577F39C7D)
}

pub unsafe fn set_low_priority_generators_active(active: bool) {
    invoke!((), 0x608207E7A8FB787C, active)
}

pub unsafe fn remove_vehicles_from_generators_in_area(start: Vector3<f32>, end: Vector3<f32>, unknown: bool) {
    invoke!((), 0x46A1E1A299EC4BBA, start, end, unknown)
}

pub unsafe fn delete(handle: &mut Handle) {
    invoke!((), 0xEA386986E786A54F, handle)
}

pub unsafe fn start_horn<H>(handle: Handle, duration: u32, mode: H, forever: bool) where H: Hashable {
    invoke!((), 0x9C8C6504B5B63D2C, handle, duration, mode.joaat(), forever)
}