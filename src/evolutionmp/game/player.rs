use super::Handle;
use crate::game::entity::Entity;

pub struct Player {
    handle: Handle
}

impl Player {
    pub fn local() -> Player {
        let handle = unsafe { crate::natives::player::get_local_handle() };
        Player { handle }
    }
}

impl Entity for Player {
    fn get_handle(&self) -> Handle {
        self.handle
    }
}