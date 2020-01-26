use std::time::{Duration, Instant};
use game::controls::{Control, Group as ControlGroup};
use game::ui::{CursorSprite, LoadingPrompt};
use winapi::um::winuser::{VK_NUMPAD5, VK_NUMPAD2, VK_NUMPAD0, VK_RIGHT, VK_LEFT, VK_BACK, ReleaseCapture};
use cgmath::{Vector3, Vector2, Matrix4, Transform};
use std::collections::{VecDeque, HashMap};
use std::sync::atomic::Ordering;
use crate::runtime::{Script, ScriptEnv, ScriptContainer, Runtime, TaskQueue};
use crate::pattern::MemoryRegion;
use crate::GameState;
use crate::{invoke, game, native};
use crate::game::entity::Entity;
use crate::game::stats::Stat;
use crate::game::ped::Ped;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::game::{streaming, gameplay, dlc, script, clock, Rgb, Rgba};
use crate::win::input::{KeyboardEvent, InputEvent};
use crate::game::streaming::{Model, AnimDict, Resource};
use crate::game::camera::Camera;
use crate::game::blip::{Blip, BlipName};
use crate::game::ui::FrontendButtons;
use crate::events::ScriptEvent;
use crate::native::pool::{Pool, Handleable};
use crate::scripts::console::ScriptConsole;
use crate::scripts::vehicle::ScriptVehicle;
use crate::scripts::cleanup::ScriptCleanWorld;
use crate::scripts::pointing::ScriptFingerPointing;
use std::sync::Mutex;
use std::rc::Rc;

pub mod console;
pub mod vehicle;
pub mod cleanup;
pub mod pointing;
//pub mod network;

pub fn init(runtime: &mut Runtime) {
    crate::info!("Registering scripts");
    //network::init(runtime);

    runtime.register_script("console", ScriptConsole::new());
    runtime.register_script("clean_world", ScriptCleanWorld::new());
    runtime.register_script("vehicle", ScriptVehicle::new());
    runtime.register_script("finger_pointing", ScriptFingerPointing::new());
    runtime.register_script("command", ScriptCommand::new());
}

pub fn command_vehicle(env: &mut ScriptEnv, args: &[String]) {
    match args {
        &[ref input] => {
            let model = Model::new(input);
            if model.is_valid() && model.is_in_cd_image() && model.is_vehicle() {
                let player = Player::local();
                let ped = player.get_ped();
                if !ped.is_in_any_vehicle(false) {
                    let veh = Vehicle::new(env, model, ped.get_position(), ped.get_heading(), false, false)
                        .expect("Vehicle creation failed");
                    ped.put_into_vehicle(&veh, -1);
                    env.log(format!("~y~Spawned vehicle ~w~{}~y~ at your position", input))
                } else {
                    env.log("~r~You're already in a vehicle");
                }
            } else {
                env.log(format!("~r~Invalid vehicle model: ~w~{}", input));
            }
        },
        _ => env.log("~r~Usage: /veh <model>")
    }
}

pub struct ScriptCommand {
    tasks: TaskQueue,
    commands: HashMap<String, Rc<Box<dyn Fn(&mut ScriptEnv, &[String])>>>
}

impl ScriptCommand {
    pub fn new() -> ScriptCommand {
        ScriptCommand {
            tasks: TaskQueue::new(),
            commands: HashMap::new()
        }
    }

    pub(crate) fn register_command<N, C>(&mut self, name: N, command: C) where N: Into<String>, C: Fn(&mut ScriptEnv, &[String]) + 'static {
        self.commands.insert(name.into(), Rc::new(Box::new(command)));
    }
}

impl Script for ScriptCommand {
    fn prepare(&mut self, env: ScriptEnv) {
        self.register_command("veh", command_vehicle);
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        self.tasks.process(&mut env);
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        match event {
            ScriptEvent::ConsoleInput(command) => {
                let mut parts = command.split(" ").map(|s|s.to_owned()).collect::<Vec<String>>();
                let name = parts.remove(0);
                if let Some(command) = self.commands.get(&name).cloned() {
                    self.tasks.push(move |env| {
                        (*command)(env, &parts[..]);
                    });
                } else {
                    self.tasks.push(move |env| {
                        env.log(format!("~r~Unknown command: {}", name));
                    });
                }
                true
            },
            _ => false
        }
    }
}