use crate::runtime::{TaskQueue, Script, ScriptEnv};
use std::collections::VecDeque;
use crate::events::ScriptEvent;
use crate::game::player::Player;
use crate::win::input::{InputEvent, KeyboardEvent};
use winapi::um::winuser::VK_NUMPAD0;
use crate::game::controls::{Group, Control};
use crate::game::entity::Entity;
use crate::game::vehicle::VehicleModel;
use crate::game::ped::Ped;

pub struct ScriptVehicle {
    tasks: TaskQueue
}

impl ScriptVehicle {
    pub fn new() -> ScriptVehicle {
        ScriptVehicle {
            tasks: TaskQueue::new()
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

    }

    fn frame(&mut self, mut env: ScriptEnv) {
        use crate::game::controls;
        let console = crate::scripts::console::is_open();

        let player = Player::local();
        let ped = player.get_ped();

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
                                self.tasks.push(move |env| {
                                    let player = Player::local();
                                    let ped = player.get_ped();

                                    if let Some(veh) = ped.get_in_vehicle(false) {
                                        veh.repair();
                                        //game::audio::play_sound_frontend(-1, "CHECKPOINT_PERFECT", "HUD_MINI_GAME_SOUNDSET", true);
                                    }
                                });
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