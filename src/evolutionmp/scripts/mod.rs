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
use std::error::Error;
use winapi::_core::str::FromStr;
use winapi::_core::any::TypeId;
use winapi::_core::fmt::{Formatter, Display};
use crate::hash::Hashable;

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

pub fn command_teleport(env: &mut ScriptEnv, args: &mut CommandArgs) -> Result<(), CommandExecutionError> {
    let pos = Vector3::new(
        args.read::<f32>()?,
        args.read::<f32>()?,
        args.read::<f32>()?
    );
    let player = Player::local();
    let ped = player.get_ped();
    ped.set_position_keep_vehicle(pos);
    Ok(())
}

pub fn command_vehicle(env: &mut ScriptEnv, args: &mut CommandArgs) -> Result<(), CommandExecutionError> {
    let model = args.read::<Model>()?;
    if model.is_valid() && model.is_in_cd_image() && model.is_vehicle() {
        let player = Player::local();
        let ped = player.get_ped();
        if !ped.is_in_any_vehicle(false) {
            let veh = Vehicle::new(env, &model, ped.get_position(), ped.get_heading(), false, false)
                .ok_or("Vehicle creation failed")?;
            ped.put_into_vehicle(&veh, -1);
            env.log(format!("~y~Spawned vehicle ~w~{}~y~ at your position", model.to_string()));
            Ok(())
        } else {
            Err("You're already in a vehicle")?
        }
    } else {
        Err(format!("Invalid vehicle model: ~w~{}", model.to_string()))?
    }
}

pub struct ScriptCommand {
    tasks: TaskQueue,
    commands: HashMap<String, Rc<Box<dyn Fn(&mut ScriptEnv, &mut CommandArgs) -> Result<(), CommandExecutionError>>>>
}

impl ScriptCommand {
    pub fn new() -> ScriptCommand {
        ScriptCommand {
            tasks: TaskQueue::new(),
            commands: HashMap::new()
        }
    }

    pub(crate) fn register_command<N, C>(&mut self, name: N, command: C)
        where N: Into<String>,
              C: Fn(&mut ScriptEnv, &mut CommandArgs) -> Result<(), CommandExecutionError> + 'static {

        self.commands.insert(name.into(), Rc::new(Box::new(command)));
    }
}

impl Script for ScriptCommand {
    fn prepare(&mut self, env: ScriptEnv) {
        self.register_command("tp", command_teleport);
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
                        let mut args = CommandArgs::new(&parts[..]);
                        if let Err(error) = (*command)(env, &mut args) {
                            env.error(format!("{}", error))
                        }
                    });
                } else {
                    self.tasks.push(move |env| {
                        env.error(format!("Unknown command: {}", name));
                    });
                }
                true
            },
            _ => false
        }
    }
}

pub enum CommandExecutionError {
    ArgParseError(CommandArgParseError),
    WrongUsage(String),
    Generic(String)
}

impl<T> From<T> for CommandExecutionError where T: AsRef<str> {
    fn from(e: T) -> Self {
        CommandExecutionError::Generic(e.as_ref().to_owned())
    }
}

impl From<CommandArgParseError> for CommandExecutionError {
    fn from(e: CommandArgParseError) -> Self {
        CommandExecutionError::ArgParseError(e)
    }
}

impl std::fmt::Display for CommandExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandExecutionError::ArgParseError(e) => {
                f.pad(&format!("Argument parse error: {}", e))
            },
            CommandExecutionError::WrongUsage(usage) => {
                f.pad(&format!("Usage: {}", usage))
            },
            CommandExecutionError::Generic(e) => f.pad(e)
        }
    }
}

pub struct CommandArgs<'a> {
    args: &'a [String],
    index: usize
}

pub enum CommandArgParseError {
    IndexOutOfBounds(usize, usize),
    Generic(String)
}

impl<T> From<T> for CommandArgParseError where T: AsRef<str> {
    fn from(e: T) -> Self {
        CommandArgParseError::Generic(e.as_ref().to_owned())
    }
}

impl std::fmt::Display for CommandArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandArgParseError::IndexOutOfBounds(index, len) => {
                f.pad(&format!("Index is out of bounds: {} (total {})", index, len))
            },
            CommandArgParseError::Generic(e) => f.pad(e)
        }
    }
}

impl<'a> CommandArgs<'a> {
    pub fn new(args: &[String]) -> CommandArgs {
        CommandArgs { args, index: 0 }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn read_str(&mut self) -> Result<&String, CommandArgParseError> {
        self.args.get(self.index).map(|a| {
            self.index += 1;
            a
        }).ok_or(CommandArgParseError::IndexOutOfBounds(self.index, self.args.len()))
    }

    pub fn read<T>(&mut self) -> Result<T, CommandArgParseError> where T: CommandArg {
        T::parse(self)
    }
}

pub trait CommandArg: Sized {
    fn parse(args: &mut CommandArgs) -> Result<Self, CommandArgParseError>;
}

impl<T, E> CommandArg for T where T: FromStr<Err=E>, E: Display {
    fn parse(args: &mut CommandArgs) -> Result<Self, CommandArgParseError> {
        Ok(args.read_str()?.parse::<T>().map_err(|e| format!("Error parsing arg from string: {}", e))?)
    }
}

impl CommandArg for Model {
    fn parse(args: &mut CommandArgs) -> Result<Self, CommandArgParseError> {
        args.read_str().map(|a| Model::new(&a))
    }
}