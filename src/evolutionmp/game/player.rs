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

    pub fn get_address(&self) -> *mut u8 {
        unsafe { (crate::native::pool::PLAYER_ADDRESS.unwrap())(self.get_handle()) }
    }

    pub fn get_ped(&self) -> Ped {
        Ped::from_player(self)
    }

    pub fn get_name<'a>(&self) -> &'a str {
        unsafe { crate::native::player::get_name(self.handle) }
    }

    pub fn disable_vehicle_rewards(&self) {
        unsafe { crate::native::player::disable_vehicle_rewards(self.handle) }
    }
}