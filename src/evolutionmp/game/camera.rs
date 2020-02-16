use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;
use cgmath::Vector3;
use crate::native::pool::Handleable;

pub struct Camera {
    handle: Handle
}

pub fn get_gameplay_relative_heading() -> f32 {
    invoke!(f32, 0x743607648ADD4587)
}

pub fn get_gameplay_relative_pitch() -> f32 {
    invoke!(f32, 0x3A6867B4845BEDA2)
}

impl Camera {
    pub fn gameplay() -> Camera {
        Self::new("gameplay", false).expect("gameplay camera missing")
    }

    pub fn new<H>(name: H, unknown: bool) -> Option<Camera> where H: Hashable {
        invoke!(Option<Camera>, 0x5E3CF89C6BCCA67D, name.joaat(), unknown)
    }

    pub fn exists(&self) -> bool {
        invoke!(bool, 0xA7A932170592B50E, self.handle)
    }

    pub fn destroy(&self, check_this_script: bool) {
        invoke!((), 0x865908C81A2C22E9, self.handle, check_this_script)
    }

    pub fn get_position(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0xBAC038F7459AE5AE, self.handle)
    }

    pub fn get_rotation(&self, order: u32) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x7D304C1C955E3E12, self.handle, order)
    }

    pub fn get_fov(&self) -> f32 {
        invoke!(f32, 0xC3330A45CCCDB26A, self.handle)
    }

    pub fn get_near_clip(&self) -> f32 {
        invoke!(f32, 0xC520A34DAFBF24B1, self.handle)
    }

    pub fn get_far_clip(&self) -> f32 {
        invoke!(f32, 0xB60A9CFEB21CA6AA, self.handle)
    }
}

crate::impl_handle!(Camera);