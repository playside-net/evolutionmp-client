use std::collections::VecDeque;
use super::{ScriptEnv, Script};
use crate::game;
use crate::game::player::Player;
use crate::game::ped::{PedBone, Ped};
use crate::game::entity::Entity;
use crate::game::{Rgba, Rgb};
use crate::game::controls::{Group as ControlGroup, Control};
use crate::events::ScriptEvent;
use cgmath::{Vector3, Zero, Array, Vector2, MetricSpace};
use std::time::Instant;
use winapi::_core::time::Duration;
use crate::game::streaming::AnimDict;
use crate::game::prop::Prop;
use crate::native::pool::Pool;
use crate::hash::Hashable;
use crate::game::ui::Font;

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

        if let Some(pave) = game::pathfind::get_nearest_pavement(ped.get_position(), true, 0) {

        }

        for prop in game::prop::get_pool().iter() {
            //if prop.get_model() == "prop_traffic_01a".joaat() {
                //prop.set_dynamic(false);
                //prop.set_light_color(false, Rgb::new(255, 0, 0));
            //}
            if prop.get_position().distance(ped.get_position()) < 25.0 {
                let model = prop.get_model();
                let raw = unsafe { std::mem::transmute(model.0) };
                let raw_name = format!("{}", model);
                let model = crate::native::OBJECT_HASHES.get(&raw).cloned().unwrap_or_else(|| raw_name.as_str());
                let scale = Vector2::from_value(0.35);
                let color = Rgba::WHITE;
                game::ui::at_origin(prop.get_position() + Vector3::unit_z() * 0.5, || {
                    game::ui::draw_text(format!("Model: {}", model), Vector2::zero(), color, Font::ChaletLondon, scale);
                });
                //game::graphics::draw_marker(0, prop.get_position(), Vector3::zero(), Vector3::zero(), Vector3::from_value(1.5), Rgba::WHITE, false, false, false, None, false);
            }
        }


        let head = ped.get_bone(PedBone::SkelHead).unwrap();
        let start = head.get_position();
        let end = ped.get_position_by_offset(Vector3::new(0.0, distance, -distance / 2.0));

        let ray = game::worldprobe::Probe::new_ray(start, end.truncate().extend(start.z), 1, &ped, 7).get_result(true);
        if ray.hit {
            game::graphics::draw_line(start, ray.end, Rgba::WHITE);
            let pos = Vector2::new(2.0, 2.0);
            let scale = Vector2::from_value(0.35);
            let color = Rgba::WHITE;
            game::ui::draw_text(format!("Material: {:?}; Entity: {:?}", ray.material, ray.entity), pos, color, Font::ChaletLondon, scale)
        }

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