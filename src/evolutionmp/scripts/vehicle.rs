use crate::runtime::{TaskQueue, Script, ScriptEnv};
use crate::events::ScriptEvent;
use crate::win::input::{InputEvent, KeyboardEvent};
use crate::game;
use crate::game::player::Player;
use crate::game::controls::{Group, Control};
use crate::game::entity::Entity;
use crate::game::vehicle::{VehicleModel, MissionTrain};
use crate::game::ped::Ped;
use crate::game::scaleform::{Scaleform, ScaleformArg};
use crate::game::{Rgba, GameState};
use winapi::um::winuser::{VK_NUMPAD0, VK_LEFT, VK_RIGHT};
use cgmath::{Vector2, Zero, Array};
use std::collections::VecDeque;
use crate::game::ui::Font;

pub struct ScriptVehicle {
    tasks: TaskQueue,
    scaleform: Option<Scaleform>
}

impl ScriptVehicle {
    pub fn new() -> ScriptVehicle {
        ScriptVehicle {
            tasks: TaskQueue::new(),
            scaleform: None
        }
    }

    fn try_enter_vehicle(&self, ped: &Ped, env: &mut ScriptEnv) -> bool {
        if !ped.is_in_any_vehicle(false) {
            if let Some(vehicle) = ped.get_entering_vehicle() {
                let model = VehicleModel::from_vehicle(&vehicle);
                let seat = ped.get_seat_is_trying_to_enter();
                if vehicle.is_seat_free(seat) {
                    ped.get_tasks().enter_vehicle(&vehicle, 5000, seat, 1.0, 1);
                    return true;
                }
            }
        }
        false
    }

    fn try_leave_vehicle(&self, ped: &Ped, env: &mut ScriptEnv) -> bool {
        if ped.is_in_any_vehicle(false) {
            let vehicle = ped.get_using_vehicle().unwrap();
            ped.get_tasks().leave_vehicle(&vehicle, 1);
            return true;
        }
        false
    }
}

impl Script for ScriptVehicle {
    fn prepare(&mut self, mut env: ScriptEnv) {
        /*let scaleform = Scaleform::new(&mut env, "BINOCULARS").unwrap();
        scaleform.invoke::<()>("SET_CAM_LOGO", &[ScaleformArg::I32(0)]);
        self.scaleform = Some(scaleform);*/
        game::audio::set_mobile_radio_enabled(true);
    }

    fn frame(&mut self, mut env: ScriptEnv, game_state: GameState) {
        use crate::game::controls;
        let console = crate::scripts::console::is_open();

        let player = Player::local();
        let ped = player.get_ped();

        /*if let Some(scaleform) = self.scaleform.as_ref() {
            scaleform.render_fullscreen(Rgba::WHITE);
        }*/

        if let Some(vehicle) = ped.get_in_vehicle(false) {
            if VehicleModel::from_vehicle(&vehicle).is_train() {
                let train = MissionTrain { vehicle };
                let scale = Vector2::from_value(0.35);
                let color = Rgba::WHITE;
                game::ui::draw_text(format!("Train node: {}", train.get_track_node()), Vector2::zero(), color, Font::ChaletLondon, scale);
            }
        }

        if ped.exists() {
            if controls::is_just_pressed(Group::Move, Control::Enter) {
                controls::disable_action(Group::Move, Control::Enter, true);
                self.try_enter_vehicle(&ped, &mut env);
            }
            if controls::is_just_pressed(Group::Move, Control::VehicleExit) {
                controls::disable_action(Group::Move, Control::VehicleExit, true);
                self.try_leave_vehicle(&ped, &mut env);
            }
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(KeyboardEvent::Key { key, is_up, .. }) => {
                        match *key {
                            VK_NUMPAD0 if !is_up => {
                                game::radio::skip_track();
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