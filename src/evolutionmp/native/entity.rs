use crate::invoke;
use crate::game::Handle;
use crate::hash::Hash;
use cgmath::Vector3;

pub unsafe fn exists(handle: Handle) -> bool {
    invoke!(bool, 0x7239B21A38F536BA, handle)
}

pub unsafe fn is_belong_to_this_script(handle: Handle, p2: bool) -> bool {
    invoke!(bool, 0xDDE6DF5AE89981D2, handle, p2)
}

pub unsafe fn has_drawable(handle: Handle) -> bool {
    invoke!(bool, 0x060D6E96F8B8E48D, handle)
}

pub unsafe fn has_physics(handle: Handle) -> bool {
    invoke!(bool, 0xDA95EA3317CC5064, handle)
}

pub unsafe fn is_dead(handle: Handle) -> bool {
    invoke!(bool, 0x5F9532F3B5CC2551, handle)
}

pub unsafe fn get_position(handle: Handle) -> Vector3<f32> {
    let alive = !is_dead(handle);
    invoke!(Vector3<f32>, 0x3FEF770D40960D5A, handle, alive)
}

pub unsafe fn set_position_no_offset(handle: Handle, pos: Vector3<f32>, axis: Vector3<bool>) {
    invoke!((), 0x239A3351AC1DA385, handle, pos, axis)
}

pub unsafe fn get_rotation(handle: Handle, order: u32) -> Vector3<f32> {
    invoke!(Vector3<f32>, 0xAFBD61CC738D9EB9, handle, order)
}

pub unsafe fn get_rotation_velocity(handle: Handle) -> Vector3<f32> {
    invoke!(Vector3<f32>, 0x213B91045D09B983, handle)
}

pub unsafe fn get_heading(handle: Handle) -> f32 {
    invoke!(f32, 0xE83D4F9BA2A38914, handle)
}

pub unsafe fn get_roll(handle: Handle) -> f32 {
    invoke!(f32, 0x831E0242595560DF, handle)
}

pub unsafe fn get_pitch(handle: Handle) -> f32 {
    invoke!(f32, 0xD45DC2893621E1FE, handle)
}

pub unsafe fn get_type(handle: Handle) -> u32 {
    invoke!(u32, 0x8ACD366038D14505, handle)
}

pub unsafe fn is_entity(handle: Handle) -> bool {
    invoke!(bool, 0x731EC8A916BD11A1, handle)
}

pub unsafe fn get_model(handle: Handle) -> Hash {
    invoke!(Hash, 0x9F47B058362C84B5, handle)
}

pub unsafe fn is_animation_finished(handle: Handle, dictionary: &str, name: &str) -> bool {
    invoke!(bool, 0x20B711662962B472, handle, dictionary, name)
}

pub unsafe fn get_health(handle: Handle) -> u32 {
    invoke!(u32, 0xEEF059FAD016D209, handle)
}
pub unsafe fn get_max_health(handle: Handle) -> u32 {
    invoke!(u32, 0x15D757606D170C3C, handle)
}

pub unsafe fn set_health(handle: Handle, health: u32) {
    invoke!((), 0x6B76DC1F3AE6E6A3, handle, health)
}

pub unsafe fn set_max_health(handle: Handle, health: u32) {
    invoke!((), 0x166E7CF68597D8B5, handle, health)
}

pub unsafe fn set_dynamic(handle: Handle, dynamic: bool) {
    invoke!((), 0x1718DE8E3F2823CA, handle, dynamic)
}

pub unsafe fn set_position_freezed(handle: Handle, freezed: bool) {
    invoke!((), 0x428CA6DBD1094446, handle, freezed)
}

pub unsafe fn set_collision(handle: Handle, collision: bool, physics: bool) {
    invoke!((), 0x1A9205C1B9EE827F, handle, collision, physics)
}

pub unsafe fn get_position_by_offset(handle: Handle, offset: Vector3<f32>) -> Vector3<f32> {
    invoke!(Vector3<f32>, 0x1899F328B0E12848, handle, offset)
}

pub unsafe fn delete(handle: &mut Handle) {
    invoke!((), 0xAE3CBE5BF394C9C9, handle)
}

pub unsafe fn set_as_mission(handle: Handle, p1: bool, p2: bool) {
    invoke!((), 0xAD738C3085FE7E11, handle, p1, p2)
}