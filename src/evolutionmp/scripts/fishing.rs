use std::collections::VecDeque;
use super::{ScriptEnv, Script};
use crate::game;
use crate::game::player::Player;
use crate::game::ped::{PedBone, Ped};
use crate::game::entity::Entity;
use crate::game::Rgba;
use crate::game::controls::{Group as ControlGroup, Control};
use crate::events::ScriptEvent;
use cgmath::{Vector3, Zero, Array};
use std::time::Instant;
use winapi::_core::time::Duration;
use crate::game::streaming::AnimDict;
use crate::game::prop::Prop;

pub struct ScriptFishing {
    catch_time: Option<Instant>,
    prop: Option<Prop>
}

impl Script for ScriptFishing {
    fn prepare(&mut self, env: ScriptEnv) {
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        let distance = 10.0;
        let player = Player::local();
        let ped = player.get_ped();
        let head = ped.get_bone(PedBone::SkelHead).unwrap();
        let start = head.get_position();
        let end = ped.get_position_by_offset(Vector3::new(0.0, distance, -distance / 2.0));
        let probe = game::water::probe(start, end);
        if let Some(pos) = probe {
            if let Some(catch_time) = self.catch_time {
                crate::scripts::console::lock_controls();
                if Instant::now() > catch_time {
                    env.log("~g~Вы поймали рыбу!");
                    self.stop_catching(&mut env, &ped);
                } else if ped.is_in_water() {
                    env.log("~r~Вы упали в воду : C");
                    self.stop_catching(&mut env, &ped);
                }
            } else if ped.get_in_vehicle(false).is_none() && !ped.is_in_water() {
                game::ui::show_help("Press ~INPUT_CONTEXT~ to start fishing", false, true, None);
                game::graphics::draw_marker(23, pos + Vector3::unit_z() * 0.2, Vector3::zero(), Vector3::zero(), Vector3::from_value(1.0), Rgba::WHITE, false, false, false, None, false);
                if game::controls::is_just_pressed(ControlGroup::Move, Control::Context) {
                    self.catch_time = Some(Instant::now() + Duration::from_secs(15));
                    let hand = ped.get_bone(PedBone::SkelLHand).unwrap();
                    let rod = Prop::new(&mut env, "prop_fishing_rod_01", Vector3::zero(), false, false, false).unwrap();
                    hand.attach(&rod, Vector3::new(0.13, 0.1, 0.01), Vector3::new(180.0, 90.0, 70.0));
                    self.prop = Some(rod);
                    ped.set_position_freezed(true);
                    let dict = AnimDict::new("amb@world_human_stand_fishing@idle_a");
                    env.wait_for_resource(&dict);
                    ped.get_tasks().play_animation(&dict, "idle_a", 8.0, -8.0, -1, 0x110001, -1.0);
                }
            }
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        false
    }
}

impl ScriptFishing {
    pub fn new() -> ScriptFishing {
        ScriptFishing {
            catch_time: None,
            prop: None
        }
    }

    fn stop_catching(&mut self, env: &mut ScriptEnv, ped: &Ped) {
        ped.set_position_freezed(false);
        ped.get_tasks().clear_immediately();
        self.catch_time = None;
        if let Some(mut rod) = self.prop.take() {
            rod.delete();
        }
    }

    fn get_water_depth(&self, pos: Vector3<f32>) -> Option<f32> {
        let ground = game::gps::get_ground_elevation(pos, false)?;
        let water = game::water::get_height(pos - Vector3::unit_z())?;
        Some(water - ground)
    }
}