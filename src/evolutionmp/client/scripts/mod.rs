use std::net::SocketAddr;

use crate::runtime::ScriptJava;
use crate::scripts::cleanup::ScriptCleanWorld;
use crate::client::scripts::fishing::ScriptFishing;
use crate::client::scripts::cleanup::ScriptWeaponStats;

pub mod cleanup;
pub mod pointing;
pub mod fishing;

pub fn init() {
    info!("Initializing scripts");

    crate::native::script::run("clean_world", ScriptCleanWorld::new());
    crate::native::script::run("weapon_stats", ScriptWeaponStats);
    crate::native::script::run("fishing", ScriptFishing::new());
    crate::native::script::run("java", ScriptJava::new());
}