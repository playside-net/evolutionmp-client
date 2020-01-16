use crate::invoke;
use crate::game::Handle;
use cgmath::Vector3;

pub fn new_ray(start: Vector3<f32>, end: Vector3<f32>, flags: u32, entity: Handle, p8: u32) -> Handle {
    invoke!(Handle, 0x377906D8A31E5586, start, end, flags, entity, p8)
}

pub fn get_result(handle: Handle, hit: &mut bool, end: &mut Vector3<f32>, surface_normal: &mut Vector3<f32>, entity: &mut Handle) -> u32 {
    invoke!(u32, 0x3D87450E15D98694, handle, hit, end, surface_normal, entity)
}