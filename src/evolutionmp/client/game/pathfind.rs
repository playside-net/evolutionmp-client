use crate::{invoke, invoke_option};
use crate::native::NativeVector3;
use cgmath::Vector3;

pub fn get_nearest_pavement(pos: Vector3<f32>, on_ground: bool, flags: i32) -> Option<Vector3<f32>> {
    let mut result = NativeVector3::zero();
    invoke_option!(result.into(), 0xB61C8E878A4199CA, pos, on_ground, &mut result, flags)
}