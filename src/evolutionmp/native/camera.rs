use crate::invoke;
use crate::game::Handle;
use crate::hash::{Hash, Hashable};
use cgmath::Vector3;

pub unsafe fn new<H>(name: H, unknown: bool) -> Handle where H: Hashable {
    invoke!(Handle, 0x5E3CF89C6BCCA67D, name.joaat(), unknown)
}

pub unsafe fn exists(handle: Handle) -> bool {
    invoke!(bool, 0xA7A932170592B50E, handle)
}

pub unsafe fn destroy(handle: Handle, check_this_script: bool) {
    invoke!((), 0x865908C81A2C22E9, handle, check_this_script)
}

pub unsafe fn get_position(handle: Handle) -> Vector3<f32> {
    invoke!(Vector3<f32>, 0xBAC038F7459AE5AE, handle)
}

pub unsafe fn get_rotation(handle: Handle, order: u32) -> Vector3<f32> {
    invoke!(Vector3<f32>, 0x7D304C1C955E3E12, handle, order)
}

pub unsafe fn get_fov(handle: Handle) -> f32 {
    invoke!(f32, 0xC3330A45CCCDB26A, handle)
}

pub unsafe fn get_near_clip(handle: Handle) -> f32 {
    invoke!(f32, 0xC520A34DAFBF24B1, handle)
}

pub unsafe fn get_far_clip(handle: Handle) -> f32 {
    invoke!(f32, 0xB60A9CFEB21CA6AA, handle)
}

pub unsafe fn get_gameplay_relative_heading() -> f32 {
    invoke!(f32, 0x743607648ADD4587)
}