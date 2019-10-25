use super::Handle;
use crate::game::entity::Entity;
use crate::game::player::Player;

pub struct Ped {
    handle: Handle
}

impl Ped {
    pub fn from_player(player: &Player) -> Ped {
        Ped {
            handle: unsafe { crate::native::player::get_ped(player.get_handle()) }
        }
    }

    pub fn local() -> Ped {
        Ped {
            handle: unsafe { crate::native::player::get_local_ped() }
        }
    }

    pub fn is_in_any_vehicle(&self, at_get_in: bool) -> bool {
        unsafe {
            crate::invoke!(bool, 0x997ABD671D25CA0B, self.handle, at_get_in)
        }
    }
}

impl Entity for Ped {
    fn get_handle(&self) -> Handle {
        self.handle
    }
}