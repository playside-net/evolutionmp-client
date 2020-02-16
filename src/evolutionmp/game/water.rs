use crate::invoke;
use crate::game::Handle;
use crate::native::pool::Handleable;
use crate::native::NativeVector3;
use cgmath::{Vector3, Vector2};

pub fn set_height(pos: Vector2<f32>, radius: f32, height: f32) {
    invoke!((), 0xC443FD757C3BA637, pos, radius, height)
}

pub fn get_height(pos: Vector3<f32>) -> Option<f32> {
    let mut result = 0.0;
    if invoke!(bool, 0xF6829842C06AE524, pos, &mut result) {
        Some(result)
    } else {
        None
    }
}

pub fn get_height_without_waves(pos: Vector3<f32>) -> Option<f32> {
    let mut result = 0.0;
    if invoke!(bool, 0x8EE6B53CE13A9794, pos, &mut result) {
        Some(result)
    } else {
        None
    }
}

pub fn get_waves_intensity() -> f32 {
    invoke!(f32, 0x2B2A2CC86778B619)
}

pub fn reset_waves_intensity() {
    invoke!((), 0x5E5E99285AE812DB)
}

pub fn set_waves_intensity(intensity: f32) {
    invoke!((), 0xB96B00E976BE977F, intensity)
}

pub fn probe(start: Vector3<f32>, end: Vector3<f32>) -> Option<Vector3<f32>> {
    let mut result = NativeVector3::zero();
    if invoke!(bool, 0xFFA5D878809819DB, start, end, &mut result) {
        Some(result.into())
    } else {
        None
    }
}

pub fn probe_height(pos: Vector3<f32>, flags: u64) -> Option<f32> {
    let mut result = 0.0;
    if invoke!(bool, 0x2B3451FA1E3142E2, pos, flags, &mut result) {
        Some(result)
    } else {
        None
    }
}

pub struct Rise {
    handle: Handle
}

impl Rise {
    pub fn new(low: Vector2<f32>, high: Vector2<f32>, height: f32) -> Rise {
        invoke!(Rise, 0xFDBF4CDBC07E1706, low, high, height)
    }

    pub fn remove(&mut self) {
        invoke!((), 0xB1252E3E59A82AAF, self.handle);
        self.handle = 0;
    }

    pub fn exists(&self) -> bool {
        self.handle != 0
    }
}

crate::impl_handle!(Rise);