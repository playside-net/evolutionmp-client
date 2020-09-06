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
use crate::game::script::wait;

pub mod cleanup;
pub mod pointing;
pub mod fishing;

pub fn init() {
    crate::info!("Registering scripts");
    //network::init();

    crate::native::script::run("clean_world", ScriptCleanWorld::new());
    crate::native::script::run("java", ScriptJava::new());
    crate::native::script::run("dummy_wait", ScriptDummyWait);
    //crate::native::script::run("finger_pointing", ScriptFingerPointing::new());
    //crate::native::script::run("fishing", ScriptFishing::new());
}

struct ScriptDummyWait;

impl Script for ScriptDummyWait {
    fn frame(&mut self) {
        wait(1000);
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        false
    }
}