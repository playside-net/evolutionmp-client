use crate::native;
use crate::game::Handle;
use crate::hash::Hashable;
use cgmath::Vector3;

pub struct Camera {
    handle: Handle
}

impl Camera {
    pub fn new<H>(name: H, unknown: bool) -> Option<Camera> where H: Hashable {
        let handle = unsafe { native::camera::new(name, unknown) };
        if handle == 0  {
            None
        } else {
            Some(Camera { handle })
        }
    }

    pub fn exists(&self) -> bool {
        unsafe { native::camera::exists(self.handle) }
    }

    pub fn destroy(&self, check_this_script: bool) {
        unsafe { native::camera::destroy(self.handle, check_this_script) }
    }

    pub fn get_position(&self) -> Vector3<f32> {
        unsafe { native::camera::get_position(self.handle) }
    }

    pub fn get_rotation(&self, order: u32) -> Vector3<f32> {
        unsafe { native::camera::get_rotation(self.handle, order) }
    }

    pub fn get_fov(&self) -> f32 {
        unsafe { native::camera::get_fov(self.handle) }
    }
}

pub fn get_gameplay_relative_heading() -> f32 {
    unsafe { native::camera::get_gameplay_relative_heading() }
}