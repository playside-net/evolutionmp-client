use crate::invoke;
use crate::game::{Handle, Vector3};
use crate::hash::Hash;

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

pub unsafe fn get_position(handle: Handle) -> Vector3 {
    invoke!(Vector3, 0x3FEF770D40960D5A, handle, !is_dead(handle))
}

pub unsafe fn get_rotation(handle: Handle, order: u32) -> Vector3 {
    invoke!(Vector3, 0xAFBD61CC738D9EB9, handle, order)
}

pub unsafe fn get_rotation_velocity(handle: Handle) -> Vector3 {
    invoke!(Vector3, 0x213B91045D09B983, handle)
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