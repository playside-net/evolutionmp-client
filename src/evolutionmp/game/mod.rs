use crate::pattern::MemoryRegion;
use crate::native::NativeStackValue;

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
pub mod object;
pub mod pickup;
pub mod decision_event;
pub mod system;
pub mod misc;

pub type Handle = u32;

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32
}

impl Rgba {
    pub const WHITE: Rgba = Rgba::new(255, 255, 255, 255);
    pub const DARK_GRAY: Rgba = Rgba::new(81, 81, 81, 255);
    pub const BLACK: Rgba = Rgba::new(0, 0, 0, 255);

    pub const fn new(r: u32, g: u32, b: u32, a: u32) -> Rgba {
        Rgba { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u32,
    pub g: u32,
    pub b: u32
}

impl Rgb {
    pub const fn new(r: u32, g: u32, b: u32) -> Rgb {
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

impl GameState {
    pub fn is_loaded(&self) -> bool {
        match self {
            GameState::MainMenu | GameState::LoadingSpMp => true,
            _ => false
        }
    }
}