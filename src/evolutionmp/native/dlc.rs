use crate::invoke;
use crate::hash::{Hash, Hashable};

pub fn is_present<H>(dlc: H) -> bool where H: Hashable {
    invoke!(bool, 0x812595A0644CE1DE, dlc.joaat())
}

pub fn load_sp_maps() {
    invoke!((), 0xD7C10C4A637992C9)
}

pub fn load_mp_maps() {
    invoke!((), 0x0888C3502DBBEEF5)
}
