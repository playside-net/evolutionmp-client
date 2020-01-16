use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;
use cgmath::Vector3;
use crate::pattern::MemoryRegion;
use std::sync::atomic::{AtomicI32, Ordering};

pub(crate) static GEARS_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static HIGH_GEAR_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static FUEL_LEVEL_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static WHEEL_SPEED_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static CURRENT_RPM_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static ACCELERATION_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static STEERING_SCALE_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static STEERING_ANGLE_OFFSET: AtomicI32 = AtomicI32::new(0);

pub unsafe fn init(mem: &MemoryRegion) {
    let address = mem.find("48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24")
        .next().expect("gear offset")
        .add(3);
    GEARS_OFFSET.store(*address.get::<i32>() + 2, Ordering::SeqCst);
    HIGH_GEAR_OFFSET.store(*address.get::<i32>() + 6, Ordering::SeqCst);

    let address = mem.find("74 ? 0F 57 C9 0F 2F 8B ? ? ? ? 73 ? F3 0F 10 83 ? ? ? ?")
        .next().expect("fuel level offset")
        .add(8);
    FUEL_LEVEL_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);

    let address = mem.find("F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?")
        .next().expect("wheel speed offset")
        .add(4);

    WHEEL_SPEED_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);

    let address = mem.find("76 03 0F 28 F0 F3 44 0F 10 93")
        .next().expect("rpm offset")
        .add(10);
    CURRENT_RPM_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);
    ACCELERATION_OFFSET.store(*address.get::<i32>() + 16, Ordering::SeqCst);

    let address = mem.find("74 0A F3 0F 11 B3 ? ? ? ? EB 25")
        .next().expect("steering offset")
        .add(6);
    STEERING_SCALE_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);
    STEERING_ANGLE_OFFSET.store(*address.get::<i32>() + 8, Ordering::SeqCst);
}

pub fn new<H>(model: H, pos: Vector3<f32>, heading: f32, is_network: bool, this_script_check: bool) -> Handle where H: Hashable {
    invoke!(Handle, 0xAF35D0D2583051B0, model.joaat(), pos, heading, is_network, this_script_check)
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

pub fn is_radio_loud(vehicle: Handle) -> bool {
    invoke!(bool, 0x032A116663A4D5AC, vehicle)
}

pub fn get_colors(vehicle: Handle, primary: &mut u32, secondary: &mut u32) {
    invoke!((), 0xA19435F193E081AC, vehicle, primary, secondary)
}

pub fn set_colors(vehicle: Handle, primary: u32, secondary: u32) {
    invoke!((), 0x4F1D4BE3A7F24601, primary, secondary)
}

pub fn repair(vehicle: Handle) {
    invoke!((), 0x115722B1B9C14C1C, vehicle)
}

pub fn repair_deformation(vehicle: Handle) {
    invoke!((), 0x953DA1E1B12C0491, vehicle)
}

pub fn place_on_ground(vehicle: Handle) {
    invoke!((), 0x49733E92263139D1, vehicle)
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

pub fn delete(handle: &mut Handle) {
    invoke!((), 0xEA386986E786A54F, handle)
}

pub fn start_horn<H>(handle: Handle, duration: u32, mode: H, forever: bool) where H: Hashable {
    invoke!((), 0x9C8C6504B5B63D2C, handle, duration, mode.joaat(), forever)
}