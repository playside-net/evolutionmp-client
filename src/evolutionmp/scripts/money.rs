use std::collections::VecDeque;
use cgmath::{Vector3, Zero, Array, MetricSpace, Vector2};
use crate::runtime::Script;
use crate::events::ScriptEvent;
use crate::game;
use crate::game::{GameState, Rgba};
use crate::game::player::Player;
use crate::game::prop::Prop;
use crate::game::entity::Entity;
use crate::native::pool::{Pool, PoolEntry};
use crate::game::ui::Font;
use crate::hash::Hashable;

pub struct ScriptMoney {

}

impl ScriptMoney {
    pub fn new() -> ScriptMoney {
        ScriptMoney {

        }
    }
}

impl Script for ScriptMoney {
    fn prepare(&mut self) {
    }

    fn frame(&mut self, game_state: GameState) {
        let player = Player::local();
        let ped = player.get_ped();

        if let Some(prop) = Prop::find_nearest(ped.get_position(), 15.0, "v_ilev_bk_vaultdoor") {
            prop.set_heading(-20.0);
            prop.set_position_freezed(true);
        }

        let pool = game::prop::get_pool();
        let ped_pos = ped.get_position();

        const GAS_PUMP: [&'static str; 6] = ["prop_gas_pump_1a", "prop_gas_pump_1b", "prop_gas_pump_1c", "prop_gas_pump_1d", "prop_gas_pump_old_2", "prop_gas_pump_old_3"];

        for prop in pool.iter().filter(|e| e.get_position().distance(ped_pos) < 15.0).flat_map(|p|p.pooled()) {
            let pos = prop.get_position();
            let model = prop.get_model();
            if GAS_PUMP.iter().any(|m|m.joaat() == model) {
                prop.set_breakable(false);
                game::graphics::draw_marker(0, pos, Vector3::zero(), Vector3::zero(), Vector3::from_value(1.5), Rgba::WHITE, false, false, false, None, false);
            }
        }

        if let Some(prop) = Prop::find_nearest(ped.get_position(), 15.0, "prop_traffic_01d") {
            prop.set_breakable(false);
            game::graphics::draw_marker(0, prop.get_position(), Vector3::zero(), Vector3::zero(), Vector3::from_value(1.5), Rgba::WHITE, false, false, false, None, false);
        }

        const ATM: [&'static str; 4] = ["prop_atm_01", "prop_atm_02", "prop_atm_03", "prop_fleeca_atm"];

        for model in ATM.iter() {
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