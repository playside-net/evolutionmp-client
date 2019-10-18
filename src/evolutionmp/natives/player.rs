use crate::invoke;
use crate::game::Handle;

pub unsafe fn get_name<'a>() -> &'a str {
    invoke!(&str, 0x6D0DE6A7B5DA71F8)
}

pub unsafe fn is_online() -> bool {
    invoke!(bool, 0xF25D331DC2627BBC)
}

pub unsafe fn get_local_handle() -> Handle {
    invoke!(Handle, 0xA5EDC40EF369B48D)
}

pub unsafe fn get_at(index: u32) -> Handle {
    invoke!(Handle, 0x41BD2A6B006AF756, index)
}

pub unsafe fn set_invincible(player: Handle, invincible: bool) {
    invoke!((), 0x239528EACDC3E7DE, player, invincible)
}

pub unsafe fn is_invincible(player: Handle) -> bool {
    invoke!(bool, 0xB721981B2B939E07, player)
}

pub unsafe fn get_ped(player: Handle) -> Handle {
    invoke!(Handle, 0x43A66C31C68491C0, player)
}

pub unsafe fn get_local_ped() -> Handle {
    invoke!(Handle, 0xD80958FC74E988A6)
}