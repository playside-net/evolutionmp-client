use crate::invoke;
use crate::game::{Hash, Handle};

pub unsafe fn new(model: Hash, x: f32, y: f32, z: f32, heading: f32, is_network: bool, this_script_check: bool) -> Handle {
    invoke!(Handle, 0xAF35D0D2583051B0, model, x, y, z, heading, is_network, this_script_check)
}