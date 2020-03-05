use std::collections::VecDeque;
use cgmath::{Vector3, Zero, Array};
use crate::runtime::{Script, ScriptEnv};
use crate::events::ScriptEvent;
use crate::game;
use crate::game::{GameState, Rgba};
use crate::game::player::Player;
use crate::game::prop::Prop;
use crate::game::entity::Entity;

pub struct ScriptMoney {

}

impl ScriptMoney {
    pub fn new() -> ScriptMoney {
        ScriptMoney {

        }
    }
}

impl Script for ScriptMoney {
    fn prepare(&mut self, env: ScriptEnv) {
    }

    fn frame(&mut self, env: ScriptEnv, game_state: GameState) {
        let player = Player::local();
        let ped = player.get_ped();

        if let Some(prop) = Prop::find_nearest(ped.get_position(), 15.0, "v_ilev_bk_vaultdoor") {
            prop.set_heading(-20.0);
            prop.set_position_freezed(true);
        }

        for model in ["prop_atm_01", "prop_atm_02", "prop_atm_03", "prop_fleeca_atm"].iter() {
            if let Some(atm) = Prop::find_nearest(ped.get_position(), 1.0, model) {
                game::graphics::draw_marker(0, atm.get_position(), Vector3::zero(), Vector3::zero(), Vector3::from_value(1.5), Rgba::WHITE, false, false, false, None, false);
                break;
            }
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        false
    }
}