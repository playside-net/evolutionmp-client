use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use jni_dynamic::{InitArgsBuilder, JavaVM, JNIVersion};
use serde_derive::{Deserialize, Serialize};

use crate::{add_dll_directory, bind_fn_detour_ip, launcher_dir};
use crate::game::ui::FrontendButtons;

pub mod audio;
pub mod entity;
pub mod player;
pub mod ped;
pub mod vehicle;
pub mod ui;
pub mod scaleform;
pub mod controls;
pub mod stats;
pub mod dlc;
pub mod streaming;
pub mod gameplay;
pub mod script;
pub mod clock;
pub mod camera;
pub mod worldprobe;
pub mod checkpoint;
pub mod pickup;
pub mod blip;
pub mod decision_event;
pub mod system;
pub mod misc;
pub mod gps;
pub mod graphics;
pub mod radio;
pub mod locale;
pub mod interior;
pub mod water;
pub mod prop;
pub mod pathfind;
pub mod door;
pub mod data;
pub mod fire;
pub mod weapon;

pub type Handle = u32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const WHITE: Rgba = Rgba::new(255, 255, 255, 255);
    pub const DARK_GRAY: Rgba = Rgba::new(81, 81, 81, 255);
    pub const BLACK: Rgba = Rgba::new(0, 0, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum GameState {
    Playing,
    Intro,
    Legals = 3,
    MainMenu = 5,
    LoadingSpMp = 6,
}

impl GameState {
    pub fn is_loaded(&self) -> bool {
        match self {
            GameState::MainMenu | GameState::LoadingSpMp => true,
            _ => false
        }
    }
}


/*extern fn run_init_state() {
    RUN_INIT_STATE()
}*/

/*externfn skip_init(stage: u32) -> bool {
    info!("skipping init {}", stage);
    SKIP_INIT(stage)
}*/

//bind_field_ip!(INIT_STATE, "BA 07 00 00 00 8D 41 FC 83 F8 01", 2, u32);
//bind_fn!(RUN_INIT_STATE, "32 DB EB 02 B3 01 E8 ? ? ? ? 48 8B", 6, fn() -> ());
//bind_fn!(SKIP_INIT, "32 DB EB 02 B3 01 E8 ? ? ? ? 48 8B", -9, fn(u32) -> bool);
bind_fn_detour_ip!(LOAD_GAME_NOW, "33 C9 E8 ? ? ? ? 8B 0D ? ? ? ? 48 8B 5C 24 ? 8D 41 FC 83 F8 01 0F 47 CF 89 0D ? ? ? ?", 2, load_game_now, fn(u8) -> u32);

pub fn hook() {
    locale::hook();
    ui::hook();
    lazy_static::initialize(&LOAD_GAME_NOW);
}

pub fn init() {
    locale::init();
    ui::init();
}

lazy_static! {
    static ref LOADED: AtomicBool = AtomicBool::new(false);
}

pub fn is_loaded() -> bool {
    LOADED.load(Ordering::SeqCst)
}

unsafe fn load_game_now(u: u8) -> u32 {
    crate::native::init();
    crate::info!("Loading game...");
    let r = LOAD_GAME_NOW(u);
    done_loading_game();
    LOADED.store(true, Ordering::SeqCst);
    r
}

fn done_loading_game() {
    dlc::load_mp_maps();

    crate::info!("Searching for script candidates...");

    let mut script_candidates = Vec::new();

    if let Ok(entries) = launcher_dir().join("scripts").read_dir() {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(ty) = entry.file_type() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if ty.is_file() && name.ends_with(".jar") {
                        script_candidates.push(name.to_string());
                    }
                }
            }
        }
    }

    crate::info!("Found {} potential vm scripts", script_candidates.len());

    let dll_path = launcher_dir().join("java/bin/server/jvm.dll");
    add_dll_directory(&dll_path);
    let args = InitArgsBuilder::new()
        .version(JNIVersion::V8)
        .option(&format!("-XX:ErrorFile={}/hs_err_pid_%p.log", launcher_dir().join("crash-reports").display()))
        .option(&format!("-Duser.dir={}", launcher_dir().display()))
        .build().expect("failed to build jvm args");
    crate::info!("Initializing VM... working dir is {:?}", std::env::current_dir());

    crate::info!("Starting VM...");
    let vm = Arc::new(JavaVM::new(&dll_path, args).expect("vm initialization failed"));
    crate::runtime::start(script_candidates, vm);

    crate::info!("Shutting down loading screen...");

    script::shutdown_loading_screen();
    camera::fade_in(5000);
}