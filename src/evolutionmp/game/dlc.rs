use crate::hash::Hashable;
use crate::native;

pub fn is_present<H>(dlc: H) -> bool where H: Hashable {
    unsafe { native::dlc::is_present(dlc.joaat()) }
}

pub fn load_sp_maps() {
    unsafe { native::dlc::load_sp_maps() }
}

pub fn load_mp_maps() {
    unsafe { native::dlc::load_mp_maps() }
}