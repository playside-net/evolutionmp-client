use crate::invoke;
use crate::game::Handle;
use crate::hash::Hash;

pub unsafe fn set_freemode_map_behavior(freemode_behavior: bool) {
    invoke!((), 0x9BAE5AD2508DF078, freemode_behavior)
}