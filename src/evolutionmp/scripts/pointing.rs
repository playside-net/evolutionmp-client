use std::collections::VecDeque;
use crate::game;
use crate::game::player::Player;
use crate::game::streaming::AnimDict;
use crate::game::camera::Camera;
use crate::events::ScriptEvent;
use super::{ScriptEnv, Script};
use crate::game::controls::{Control, Group as ControlGroup};
use crate::game::entity::Entity;

pub struct ScriptFingerPointing {
    active: bool,
    camera: Option<Camera>
}

impl ScriptFingerPointing {
    pub fn new() -> ScriptFingerPointing {
        ScriptFingerPointing {
            active: false,
            camera: None
        }
    }
}

impl Script for ScriptFingerPointing {
    fn prepare(&mut self, mut env: ScriptEnv) {
        self.camera = Some(Camera::new("gameplay", false).expect("Camera creation failed"));
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        let player = Player::local().get_ped();
        let tasks = player.get_tasks().get_network();

        tasks.is_move_active();

        let pitch = (self.get_relative_pitch().min(42.0).max(-70.0) + 70.0) / 112.0;
        let heading = (game::camera::get_gameplay_relative_heading().min(180.0).max(-180.0) + 180.0) / 360.0;

        tasks.set_move_signal("Pitch", pitch);
        tasks.set_move_signal("Heading", heading * -1.0 + 1.0);
        tasks.set_move_signal("isBlocked", false);
        use crate::invoke;
        let first_person = invoke!(u32, 0xEE778F8C7E1142E2, invoke!(u32, 0x19CAFA3C87F7C2FF)) == 4;
        tasks.set_move_signal("isFirstPerson", first_person);

        if game::controls::is_disabled_pressed(ControlGroup::Move, Control::Cover) && !player.is_in_any_vehicle(false) {
            if !self.active {
                self.active = true;
                let dict = AnimDict::new("anim@mp_point");
                env.wait_for_resource(&dict);
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
        let camera_rotation = self.camera.as_ref().expect("missing gameplay camera").get_rotation(2);
        camera_rotation.x - Player::local().get_ped().get_pitch()
    }
}