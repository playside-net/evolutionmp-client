use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;

pub unsafe fn get_name<'a>(player: Handle) -> &'a str {
    invoke!(&str, 0x6D0DE6A7B5DA71F8, player)
}

pub unsafe fn is_online() -> bool {
    invoke!(bool, 0xF25D331DC2627BBC)
}

pub unsafe fn get_local_handle() -> Handle {
    invoke!(Handle, 0x4F8644AF03D0E0D6)
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

pub unsafe fn is_dead(player: Handle) -> bool {
    invoke!(bool, 0x424D4687FA1E5652, player)
}

pub unsafe fn is_pressing_horn(player: Handle) -> bool {
    invoke!(bool, 0xFA1E2BF8B10598F9, player)
}

pub unsafe fn set_max_wanted_level(max_level: u32) {
    invoke!((), 0xAA5F02DB48D704B9, max_level)
}

pub unsafe fn disable_vehicle_rewards(player: Handle) {
    invoke!((), 0xC142BE3BB9CE125F, player)
}

pub unsafe fn set_model<H>(player: Handle, model: H) where H: Hashable {
    invoke!((), 0x00A1CADD00108836, player, model.joaat())
}