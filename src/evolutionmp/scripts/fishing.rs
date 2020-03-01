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
use std::time::Duration;
use crate::game::streaming::AnimDict;
use crate::game::prop::Prop;
use crate::native::pool::Pool;
use crate::hash::{Hashable, Hash};
use crate::game::ui::{Font, LoadingPrompt};
use crate::game::door::Door;
use std::cmp::Ordering::Equal;
use crate::game::camera::GameplayCamera;

pub struct ScriptFishing {
    catch_time: Option<Instant>,
    prop: Option<Prop>
}

impl Script for ScriptFishing {
    fn prepare(&mut self, env: ScriptEnv) {
        fn set_door_locked(name: &str, position: Vector3<f32>, locked: bool) {
            Door::new(name)
                .set_locked(position, locked, Vector3::new(0.0, 50.0, 0.0))
        }
        set_door_locked("hei_prop_hei_bankdoor_new", Vector3::new(232.6054, 214.1584, 106.4049), true);
        set_door_locked("hei_prop_hei_bankdoor_new", Vector3::new(231.5123, 216.5177, 106.4049), true);
        set_door_locked("v_ilev_bk_door", Vector3::new(256.9125, 206.8366, 109.2830), false);
        set_door_locked("v_ilev_bk_door", Vector3::new(265.6144, 217.7971, 109.2830), false);
        set_door_locked("v_ilev_shrfdoor", Vector3::new(1855.5922, 3683.8213, 34.8928), false);
        set_door_locked("v_ilev_shrf2door", Vector3::new(-442.73795, 6015.3564, 32.2838), false);
        set_door_locked("v_ilev_shrf2door", Vector3::new(-444.43552, 6017.0537, 32.3005), false);
        set_door_locked("v_ilev_bank4door02", Vector3::new(-111.39079, 6463.931, 32.2215), false);
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        let distance = 10.0;
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

        /*let cameras = game::camera::get_pool();

        game::ui::show_loading_prompt(LoadingPrompt::SavingLeft, &format!("Cameras: {}/{}", cameras.iter().count(), cameras.capacity()));*/

        let dist = |prop: &Prop| -> f32 {
            prop.get_position().distance(ped.get_position())
        };

        /*if let Some(nearest) = props.iter().min_by(|a, b| dist(a).partial_cmp(&dist(b)).unwrap_or(Equal)) {
            let model = nearest.get_model();
            let scale = Vector2::from_value(0.35);
            let color = Rgba::WHITE;
            game::ui::at_origin(nearest.get_position() + Vector3::unit_z() * 0.5, || {
                game::ui::draw_text(format!("Model: {}", model), Vector2::zero(), color, Font::ChaletLondon, scale);
            });
            game::graphics::draw_marker(0, nearest.get_position(), Vector3::zero(), Vector3::zero(), Vector3::from_value(1.5), color, false, false, false, None, false);
        }*/

        /*let props = game::prop::get_pool();

        for prop in props.iter() {
            //if prop.get_model() == "prop_traffic_01a".joaat() {
                //prop.set_dynamic(false);
                //prop.set_light_color(false, Rgb::new(255, 0, 0));
            //}
            if prop.get_position().distance(ped.get_position()) < 25.0 {
                let model = prop.get_model();
                let scale = Vector2::from_value(0.35);
                let color = Rgba::WHITE;
                game::ui::at_origin(prop.get_position() + Vector3::unit_z() * 0.5, || {
                    game::ui::draw_text(format!("Model: {}", model), Vector2::zero(), color, Font::ChaletLondon, scale);
                });
                game::graphics::draw_marker(0, prop.get_position(), Vector3::zero(), Vector3::zero(), Vector3::from_value(1.5), Rgba::WHITE, false, false, false, None, false);
            }
        }*/

        let cam = GameplayCamera;
        let start = cam.get_position();
        let dir = cam.get_direction();
        let end = start + dir * distance;

        let ray = game::worldprobe::Probe::new_ray(start, end, 2 + 4 + 8 + 16, &ped, 7).get_result(true);
        if ray.hit {
            game::graphics::draw_line(start, ray.end, Rgba::WHITE);
            let pos = Vector2::new(2.0, 2.0);
            let scale = Vector2::from_value(0.35);
            let color = Rgba::WHITE;

            if let Some(entity) = ray.entity {
                let model = entity.get_model();
                game::ui::draw_text(format!("Model {}; pos: {:?}", model, ray.end), pos, color, Font::ChaletLondon, scale);
            }
        }

        /*if let Some(veh) = ped.get_in_vehicle(false) {
            let pos = Vector2::new(2.0, 2.0);
            let scale = Vector2::from_value(0.35);
            let color = Rgba::WHITE;
            game::ui::draw_text(format!(
                "Gear: {}\n\
                High gear: {}\n\
                Wheel speed: {}\n\
                Current RPM: {}\n\
                Acceleration: {}\n\
                Steering angle: {:?}\n\
                Steering scale: {}\n\
                Gears: {}\n\
                Clutch: {}\n\
                Turbo: {}\n\
                Rotation: {:?}",
                veh.get_current_gear(),
                veh.get_high_gear(),
                veh.get_wheel_speed(),
                veh.get_current_rpm(),
                veh.get_acceleration(),
                veh.get_steering_angle(),
                veh.get_steering_scale(),
                veh.get_gears(),
                veh.get_clutch(),
                veh.get_turbo(),
                veh.get_rotation(2),
            ), pos, color, Font::ChaletLondon, scale);
        }*/

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