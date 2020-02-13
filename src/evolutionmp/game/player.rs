use super::Handle;
use crate::invoke;
use crate::native::pool;
use crate::game::entity::Entity;
use crate::game::ped::Ped;
use crate::hash::Hashable;
use crate::game::streaming::Model;
use crate::runtime::ScriptEnv;
use crate::native::pool::Handleable;

pub struct Player {
    handle: Handle
}

pub fn get_social_club<'a>() -> &'a str {
    invoke!(&str, 0x198D161F458ECC7F)
}

pub fn is_online() -> bool {
    invoke!(bool, 0xF25D331DC2627BBC)
}

pub fn get_at(index: u32) -> Handle {
    invoke!(Handle, 0x41BD2A6B006AF756, index)
}

pub fn set_max_wanted_level(max_level: u32) {
    invoke!((), 0xAA5F02DB48D704B9, max_level)
}

impl Player {
    pub fn local() -> Player {
        invoke!(Player, 0x4F8644AF03D0E0D6)
    }

    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    pub fn get_address(&self) -> *mut u8 {
        (pool::PLAYER_ADDRESS.get().unwrap())(self.get_handle())
    }

    pub fn get_ped(&self) -> Ped {
        Ped::from_player(self)
    }

    pub fn get_name<'a>(&self) -> &'a str {
        invoke!(&str, 0x6D0DE6A7B5DA71F8, self.handle)
    }

    pub fn set_model<H>(&self, env: &mut ScriptEnv, model: H) -> bool where H: Hashable {
        let model = Model::from(model);
        if model.is_in_cd_image() && model.is_valid() {
            env.wait_for_resource(&model);
            invoke!((), 0x00A1CADD00108836, self.handle, model.joaat());
            let ped = self.get_ped();
            ped.set_default_component_variation();
            true
        } else {
            false
        }
    }

    pub fn is_invincible(&self) -> bool {
        invoke!(bool, 0xB721981B2B939E07, self.handle)
    }

    pub fn set_invincible(&self, invincible: bool) {
        invoke!((), 0x239528EACDC3E7DE, self.handle, invincible)
    }

    pub fn is_dead(&self) -> bool {
        invoke!(bool, 0x424D4687FA1E5652, self.handle)
    }

    pub fn is_pressing_horn(&self) -> bool {
        invoke!(bool, 0xFA1E2BF8B10598F9, self.handle)
    }

    pub fn disable_vehicle_rewards(&self) {
        invoke!((), 0xC142BE3BB9CE125F, self.handle)
    }
}

impl Handleable for Player {
    fn from_handle(handle: Handle) -> Option<Self> {
        Some(Player { handle })
    }

    fn get_handle(&self) -> Handle {
        self.handle
    }
}