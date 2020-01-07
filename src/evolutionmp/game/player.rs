use super::Handle;
use crate::native;
use crate::game::entity::Entity;
use crate::game::ped::Ped;
use crate::hash::Hashable;
use crate::game::streaming::Model;
use crate::runtime::ScriptEnv;

pub struct Player {
    handle: Handle
}

impl Player {
    pub fn local() -> Player {
        let handle = unsafe { native::player::get_local_handle() };
        Player { handle }
    }

    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    pub fn get_address(&self) -> *mut u8 {
        unsafe { (native::pool::PLAYER_ADDRESS.unwrap())(self.get_handle()) }
    }

    pub fn get_ped(&self) -> Ped {
        Ped::from_player(self)
    }

    pub fn get_name<'a>(&self) -> &'a str {
        unsafe { native::player::get_name(self.handle) }
    }

    pub fn disable_vehicle_rewards(&self) {
        unsafe { native::player::disable_vehicle_rewards(self.handle) }
    }

    pub fn set_model<H>(&self, env: &mut ScriptEnv, model: H) -> bool where H: Hashable {
        let model = Model::new(model);
        if model.is_in_cd_image() && model.is_valid() {
            model.request_and_wait(env);
            unsafe { native::player::set_model(self.handle, model); }
            model.mark_unused();
            true
        } else {
            false
        }
    }
}