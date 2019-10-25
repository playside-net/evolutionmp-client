use super::Handle;
use crate::game::entity::Entity;
use crate::game::ped::Ped;

pub struct Player {
    handle: Handle
}

impl Player {
    pub fn local() -> Player {
        let handle = unsafe { crate::native::player::get_local_handle() };
        Player { handle }
    }

    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    pub fn get_ped(&self) -> Ped {
        Ped::from_player(self)
    }
}