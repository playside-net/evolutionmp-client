use crate::runtime::ScriptJava;
use crate::scripts::cleanup::ScriptCleanWorld;

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