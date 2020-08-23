use std::collections::VecDeque;
use cgmath::{Vector2, Array};
use super::Script;
use crate::game;
use crate::game::player::Player;
use crate::game::streaming::{AnimDict, Resource};
use crate::game::camera::{Camera, GameplayCamera};
use crate::game::controls::{Control, Group as ControlGroup};
use crate::game::entity::Entity;
use crate::events::ScriptEvent;
use crate::native::pool::Handleable;
use crate::game::{Rgba, GameState};
use crate::game::ui::Font;

pub struct ScriptFingerPointing {
    active: bool
}

impl ScriptFingerPointing {
    pub fn new() -> ScriptFingerPointing {
        ScriptFingerPointing {
            active: false
        }
    }
}

impl Script for ScriptFingerPointing {
    fn frame(&mut self, game_state: GameState) {
        let player = Player::local().get_ped();
        let tasks = player.get_tasks().get_network();

        tasks.is_move_active();

        let pitch = (self.get_relative_pitch().min(42.0).max(-70.0) + 70.0) / 112.0;
        let heading = (GameplayCamera.get_relative_heading().min(180.0).max(-180.0) + 180.0) / 360.0;

        tasks.set_move_signal("Pitch", pitch);
        tasks.set_move_signal("Heading", heading * -1.0 + 1.0);
        tasks.set_move_signal("isBlocked", false);
        let first_person = game::camera::get_view_mode(game::camera::get_camera_type()) == 4;
        tasks.set_move_signal("isFirstPerson", first_person);

        if game::controls::is_disabled_pressed(ControlGroup::Move, Control::Cover) && !player.is_in_any_vehicle(false) {
            if !self.active {
                self.active = true;
                let dict = AnimDict::new("anim@mp_point");
                dict.request_and_wait();
                player.set_config_flag(36, true);
                tasks.do_move("task_mp_pointing", 0.5, false, &dict, 24);
            }
        } else if self.active {
            player.get_tasks().get_network().request_move_state_transition("Stop");
            player.get_tasks().clear_secondary();
            self.active = false;
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        false
    }
}

impl ScriptFingerPointing {
    fn get_relative_pitch(&self) -> f32 {
        let camera_rotation = GameplayCamera.get_rotation(2);
        camera_rotation.x - Player::local().get_ped().get_pitch()
    }
}