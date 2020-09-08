use std::net::SocketAddr;

use crate::runtime::ScriptJava;
use crate::scripts::cleanup::ScriptCleanWorld;
use crate::scripts::network::ScriptNetwork;

pub mod cleanup;
pub mod pointing;
pub mod fishing;
pub mod network;

pub fn init(server: SocketAddr) {
    crate::info!("Initializing scripts");

    crate::native::script::run("clean_world", ScriptCleanWorld::new());
    crate::native::script::run("java", ScriptJava::new());
    crate::native::script::run("net", ScriptNetwork::new(server));
}