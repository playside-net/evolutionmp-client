use crate::native;

pub fn set_freemode_map_behavior(freemode_behavior: bool) {
    unsafe { native::gameplay::set_freemode_map_behavior(freemode_behavior) }
}

pub fn set_time_scale(scale: f32) {
    unsafe { native::gameplay::set_time_scale(scale) }
}