use crate::invoke;
use crate::hash::Hash;

pub unsafe fn is_present(dlc: Hash) -> bool {
    invoke!(bool, 0x812595A0644CE1DE, dlc)
}

pub unsafe fn load_sp_maps() {
    invoke!((), 0xD7C10C4A637992C9)
}

pub unsafe fn load_mp_maps() {
    invoke!((), 0x0888C3502DBBEEF5)
}
