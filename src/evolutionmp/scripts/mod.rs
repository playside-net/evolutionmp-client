use game::controls::{Control, Group as ControlGroup};
use game::ui::{CursorSprite, LoadingPrompt};
use cgmath::{Vector3, Vector2, Matrix4, Transform, Zero, Euler, Quaternion, Matrix3, Array};
use crate::runtime::{Script, TaskQueue, ScriptJava};
use crate::pattern::MemoryRegion;
use crate::GameState;
use crate::{invoke, game, native};
use crate::game::entity::Entity;
use crate::game::stats::Stat;
use crate::game::ped::{Ped, PedBone};
use crate::game::player::Player;
use crate::game::vehicle::{Vehicle, MissionTrain};
use crate::game::{streaming, gameplay, dlc, script, clock, Rgb, Rgba};
use crate::win::input::{KeyboardEvent, InputEvent};
use crate::game::streaming::{Model, AnimDict, Resource};
use crate::game::camera::Camera;
use crate::game::blip::{Blip, BlipName};
use crate::game::ui::FrontendButtons;
use crate::events::ScriptEvent;
use crate::native::pool::{Pool, Handleable};
use crate::scripts::cleanup::ScriptCleanWorld;
use crate::scripts::pointing::ScriptFingerPointing;
use crate::hash::{Hashable, Hash};
use std::sync::Mutex;
use std::rc::Rc;
use std::error::Error;
use std::str::FromStr;
use std::any::TypeId;
use std::fmt::{Formatter, Display};
use std::time::{Duration, Instant};
use std::collections::{VecDeque, HashMap};
use std::sync::atomic::Ordering;
use crate::game::fire::ExplosionSource;

pub mod cleanup;
pub mod pointing;
pub mod fishing;

pub fn init() {
    crate::info!("Registering scripts");
    //network::init();

    crate::native::script::run("clean_world", ScriptCleanWorld::new());
    crate::native::script::run("java", ScriptJava::new());
    //crate::native::script::run("finger_pointing", ScriptFingerPointing::new());
    //crate::native::script::run("fishing", ScriptFishing::new());
}

pub fn command_train(args: &mut CommandArgs) -> Result<(), CommandExecutionError> {
    let player = Player::local();
    let ped = player.get_ped();
    let model = args.read::<u8>()?;
    let direction = args.read::<bool>()?;
    if !ped.is_in_any_vehicle(false) {
        let train = MissionTrain::new(model, ped.get_position(), direction)
            .ok_or("Train creation failed")?;
        ped.put_into_vehicle(train.as_vehicle(), -1);
        Ok(())
    } else {
        Err("You're already in a vehicle")?
    }
}

pub fn command_timecycle(args: &mut CommandArgs) -> Result<(), CommandExecutionError> {
    let name = args.read::<String>()?;
    if name == "reset" {
        game::graphics::timecycle::clear_primary_modifier();
    } else {
        let strength = args.read::<f32>()?;
        game::graphics::timecycle::set_primary_modifier(&name);
        game::graphics::timecycle::set_primary_modifier_strength(strength);
    }
    Ok(())
}

pub fn command_explosion(args: &mut CommandArgs) -> Result<(), CommandExecutionError> {
    let player = Player::local();
    let ped = player.get_ped();
    game::fire::explode(ped.get_position(), ExplosionSource::GasTank, 1.0, true, false, true);
    Ok(())
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
        T::parse(self.read_str()?)
    }

    pub fn read_coord(&mut self, origin: f32) -> Result<f32, CommandArgParseError> {
        let coord = self.read_str()?;
        if coord.starts_with("~") {
            <f32 as CommandArg>::parse(&coord[1..]).map(|c| c + origin)
        } else {
            <f32 as CommandArg>::parse(&coord)
        }
    }

    pub fn read_pos(&mut self) -> Result<Vector3<f32>, CommandArgParseError> {
        let player = Player::local();
        let ped = player.get_ped();
        let origin = ped.get_position();
        let x = self.read_coord(origin.x)?;
        let y = self.read_coord(origin.y)?;
        let z = self.read_coord(origin.z)?;
        Ok(Vector3::new(x, y, z))
    }
}

pub trait CommandArg: Sized {
    fn parse(arg: &str) -> Result<Self, CommandArgParseError>;
}

impl<T, E> CommandArg for T where T: FromStr<Err=E>, E: Display {
    fn parse(arg: &str) -> Result<Self, CommandArgParseError> {
        Ok(arg.parse::<T>().map_err(|e| format!("Error parsing arg from string: {}", e))?)
    }
}

impl CommandArg for Model {
    fn parse(arg: &str) -> Result<Self, CommandArgParseError> {
        Ok(if let Ok(hash) = arg.parse::<u32>() {
            Model::from(Hash(hash))
        } else {
            Model::from(&arg)
        })
    }
}