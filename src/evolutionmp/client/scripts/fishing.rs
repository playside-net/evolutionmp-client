use std::time::Duration;
use std::time::Instant;

use cgmath::{Array, Vector2, Vector3, Zero};

use crate::events::ScriptEvent;
use crate::game;
use crate::game::camera::GameplayCamera;
use crate::game::controls::{Control, Group as ControlGroup};
use crate::game::entity::Entity;
use crate::game::ped::{Ped, PedBone};
use crate::game::player::Player;
use crate::game::prop::Prop;
use crate::game::Rgba;
use crate::game::streaming::{AnimDict, Resource};
use crate::runtime::Script;
use crate::game::ui::{Font, FrontendButtons};

pub struct ScriptFishing {
    catch_time: Option<Instant>,
    prop: Option<Prop>,
}

impl Script for ScriptFishing {
    fn frame(&mut self) {
        if crate::game::is_loaded() {
            //let result = crate::game::ui::warn("Hello", "Line1", "Line2", FrontendButtons::OK | FrontendButtons::CANCEL, true);

            //info!("Clicked {:?}", result);
        }

        let distance = 10.0;
        let player = Player::local();
        let ped = player.get_ped();

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

            game::ui::draw_text(format!("{:?}", ray), pos, color, Font::ChaletLondon, scale);
            if let Some(mut entity) = ray.entity {
                let model = entity.get_model();
                game::ui::draw_text(format!("Model {}; pos: {:?}", model, ray.end), pos + Vector2::unit_y() * 35.0, color, Font::ChaletLondon, scale);

                if game::controls::is_disabled_just_pressed(ControlGroup::Move, Control::Cover) {
                    /*if let Some(int) = Interior::from_pos(ped.get_position()) {
                        warn!("{:?}", int.get_info());
                    }*/
                    warn!("MODEL: {:?} POS: {:?}", model, entity.get_position());
                    if let Some(prop) = entity.as_prop() {
                        if prop.is_broken() {
                            prop.place_on_ground_properly()
                        }
                    }
                    //entity.delete()
                }
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
                //crate::scripts::console::lock_controls();
                if Instant::now() > catch_time {
                    self.stop_catching(&ped);
                } else if ped.is_in_water() {
                    self.stop_catching(&ped);
                }
            } else if ped.get_in_vehicle(false).is_none() && !ped.is_in_water() {
                game::ui::show_help("Press ~INPUT_CONTEXT~ to start fishing", false, true, None);
                game::graphics::draw_marker(23, pos + Vector3::unit_z() * 0.2, Vector3::zero(), Vector3::zero(), Vector3::from_value(1.0), Rgba::WHITE, false, false, false, None, false);
                if game::controls::is_just_pressed(ControlGroup::Move, Control::Context) {
                    self.catch_time = Some(Instant::now() + Duration::from_secs(15));
                    let hand = ped.get_bone(PedBone::SkelLHand).unwrap();
                    let rod = Prop::new("prop_fishing_rod_01", Vector3::zero(), false, false, false).unwrap();
                    hand.attach(&rod, Vector3::new(0.13, 0.1, 0.01), Vector3::new(180.0, 90.0, 70.0));
                    self.prop = Some(rod);
                    ped.set_position_freezed(true);
                    let dict = AnimDict::new("amb@world_human_stand_fishing@idle_a");
                    dict.request_and_wait();
                    ped.get_tasks().play_animation(&dict, "idle_a", 8.0, -8.0, -1, 0x110001, -1.0);
                }
            }
        }
    }

    fn event(&mut self, _event: ScriptEvent) {}
}

impl ScriptFishing {
    pub fn new() -> ScriptFishing {
        ScriptFishing {
            catch_time: None,
            prop: None,
        }
    }

    fn stop_catching(&mut self, ped: &Ped) {
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