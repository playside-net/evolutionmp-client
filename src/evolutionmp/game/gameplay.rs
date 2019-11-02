use crate::native;

pub fn set_freemode_map_behavior(freemode_behavior: bool) {
    unsafe { native::gameplay::set_freemode_map_behavior(freemode_behavior) }
}