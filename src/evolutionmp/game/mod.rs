use crate::pattern::MemoryRegion;
use crate::native::NativeStackValue;
use winapi::_core::time::Duration;

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

pub type Handle = u32;

#[derive(Debug, Copy, Clone)]
pub struct Vector3<T> where T: NativeStackValue + Copy + Clone {
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T> Vector3<T> where T: NativeStackValue + Copy + Clone {
    pub fn new(x: T, y: T, z: T) -> Vector3<T> {
        Vector3 { x, y, z }
    }

    pub fn union(value: T) -> Vector3<T> {
        Self::new(value, value, value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vector2<T> where T: NativeStackValue + Copy + Clone {
    pub x: T,
    pub y: T
}

impl<T> Vector2<T> where T: NativeStackValue + Copy + Clone {
    pub fn new(x: T, y: T) -> Vector2<T> {
        Vector2 { x, y }
    }

    pub fn union(value: T) -> Vector2<T> {
        Self::new(value, value)
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

impl GameState {
    pub fn is_loaded(&self) -> bool {
        match self {
            GameState::MainMenu | GameState::LoadingSpMp => true,
            _ => false
        }
    }
}