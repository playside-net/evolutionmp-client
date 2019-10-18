use crate::invoke;
use crate::game::{Hash, Handle, Vector3};

pub unsafe fn new(model: Hash, pos: Vector3, heading: f32, is_network: bool, this_script_check: bool) -> Handle {
    invoke!(Handle, 0xAF35D0D2583051B0, model, pos, heading, is_network, this_script_check)
}