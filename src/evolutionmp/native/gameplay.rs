use crate::invoke;
use crate::game::Handle;
use crate::hash::Hash;

pub fn set_freemode_map_behavior(freemode_behavior: bool) {
    invoke!((), 0x9BAE5AD2508DF078, freemode_behavior)
}

pub fn set_time_scale(scale: f32) {
    invoke!((), 0x1D408577D440E81E, scale)
}