use crate::win::input::{InputEvent, KeyboardEvent};
use crate::runtime::{Script, ScriptEnv, TaskQueue};
use crate::events::ScriptEvent;
use crate::game::camera::{Camera, GameplayCamera, CameraType};
use crate::game::player::Player;
use crate::game::entity::Entity;
use crate::game;
use crate::game::controls::{Group, Control as InputControl};
use std::collections::VecDeque;
use cgmath::{Vector3, Array, Vector2, Zero};
use winapi::um::winuser::{VK_F2, VK_SHIFT, VK_CONTROL};
use crate::game::scaleform::Scaleform;

pub struct ScriptFly {
    camera: Option<Camera>,
    shift: bool,
    ctrl: bool,
    down: bool,
    up: bool
}

impl ScriptFly {
    pub fn new() -> ScriptFly {
        ScriptFly {
            camera: None,
            shift: false,
            ctrl: false,
            down: false,
            up: false
        }
    }
}

impl Script for ScriptFly {
    fn prepare(&mut self, mut env: ScriptEnv) {

    }

    fn frame(&mut self, mut env: ScriptEnv) {
        if !crate::scripts::console::is_open() {
            if let Some(camera) = self.camera.as_ref() {
                let pos = camera.get_position();
                let rot = camera.get_rotation(2);
                let dir = camera.get_direction();
                let speed = (if self.shift { 3.0 } else { 1.0 }) * (if self.ctrl { 0.5 } else { 1.0 });

                let delta_heading = game::controls::get_disabled_normal(Group::Move, InputControl::ScriptRightAxisX);
                let delta_pitch = game::controls::get_disabled_normal(Group::Move, InputControl::ScriptRightAxisY);
                let move_side = game::controls::get_disabled_normal(Group::Move, InputControl::ScriptLeftAxisX);
                let move_front = game::controls::get_disabled_normal(Group::Move, InputControl::ScriptLeftAxisY);
                let move_up = (if self.up { 0.5 } else { 0.0 }) - (if self.down { 0.5 } else { 0.0 });

                let velocity = dir * (move_front * speed);
                let up = Vector3::unit_z();
                let right = dir.cross(up) * move_side * 0.5 * speed;

                let player = Player::local();
                let ped = player.get_ped();
                ped.set_position_no_offset(pos + velocity + Vector3::from_value(1.0), Vector3::from_value(false));
                ped.set_heading(rot.z);
                camera.set_position(pos - velocity + right - up * move_up * speed);
                camera.set_rotation(Vector3::new(
                    rot.x + delta_pitch * -5.0,
                    0.0,
                    rot.z + delta_heading * -5.0
                ), 2);
            }
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(event) => {
                        match event {
                            KeyboardEvent::Key { key, is_up, .. } => {
                                if *key == VK_F2 && !*is_up {
                                    let player = Player::local();
                                    let ped = player.get_ped();
                                    if let Some(camera) = self.camera.take() {
                                        ped.set_position_no_offset(camera.get_position(), Vector3::from_value(false));
                                        camera.destroy(false);
                                        game::camera::render_scripted(false, None);
                                        ped.set_position_freezed(false);
                                        ped.set_invincible(false);
                                        ped.set_visible(true);
                                        ped.set_collision(true, true);
                                    } else {
                                        let rot = GameplayCamera.get_rotation(2);
                                        let pos = ped.get_position();
                                        let camera = Camera::new_parameterized(CameraType::DefaultScripted, pos, rot, 45.0).unwrap();
                                        camera.set_active(true);
                                        self.camera = Some(camera);
                                        game::camera::render_scripted(true, None);
                                        ped.set_position_freezed(true);
                                        ped.set_invincible(true);
                                        ped.set_visible(false);
                                        ped.set_collision(false, false);
                                    }
                                    return true;
                                } else if *key == VK_SHIFT {
                                    self.shift = !*is_up;
                                } else if *key == VK_CONTROL {
                                    self.ctrl = !*is_up;
                                } else if *key == 0x51 /*Q*/ {
                                    self.up = !*is_up;
                                } else if *key == 0x45 /*E*/ {
                                    self.down = !*is_up;
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        false
    }
}