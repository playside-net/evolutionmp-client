use crate::runtime::ScriptJava;
use crate::scripts::cleanup::ScriptCleanWorld;
use crate::scripts::network::ScriptNetwork;
use std::net::SocketAddr;

pub mod cleanup;
pub mod pointing;
pub mod fishing;
pub mod network;

pub fn init(server: SocketAddr) {
    crate::info!("Registering scripts");

    crate::native::script::run("clean_world", ScriptCleanWorld::new());
    crate::native::script::run("java", ScriptJava::new());
    crate::native::script::run("net", ScriptNetwork::new(server));
}