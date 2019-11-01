use crate::pattern::MemoryRegion;

pub mod entity;
pub mod player;
pub mod ped;
pub mod vehicle;
pub mod ui;
pub mod scaleform;
pub mod controls;
pub mod stats;

pub(crate) static mut GAME_STATE: *const GameState = std::ptr::null_mut();

pub unsafe fn init(mem: &MemoryRegion) {
    GAME_STATE = mem.find_first_await("83 3D ? ? ? ? ? 8A D9 74 0A", 50, 1000)
        .expect("game state")
        .add(2)
        .read_ptr(5)
        .get::<GameState>();
}

pub type Handle = u32;

#[derive(Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }
}

#[derive(Debug)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Rgba {
    pub const WHITE: Rgba = Rgba::new(1.0, 1.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl Rgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Rgb {
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
    LoadingSpMp = 6
}

pub fn get_state() -> GameState {
    unsafe { *GAME_STATE }
}