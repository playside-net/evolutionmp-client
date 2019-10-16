use super::Handle;
use crate::game::{Hash, Vector3};
use std::ffi::{CString, CStr};

pub trait Entity {
    fn get_handle(&self) -> Handle;

    fn exists(&self) -> bool {
        unsafe { crate::natives::entity::exists(self.get_handle()) }
    }

    fn is_dead(&self) -> bool {
        unsafe { crate::natives::entity::is_dead(self.get_handle()) }
    }

    fn get_position(&self) -> Vector3 {
        unsafe { crate::natives::entity::get_position(self.get_handle()) }
    }

    fn get_rotation(&self, order: u32) -> Vector3 {
        unsafe { crate::natives::entity::get_rotation(self.get_handle(), order) }
    }

    fn get_rotation_velocity(&self) -> Vector3 {
        unsafe { crate::natives::entity::get_rotation_velocity(self.get_handle()) }
    }

    fn get_heading(&self) -> f32 {
        unsafe { crate::natives::entity::get_heading(self.get_handle()) }
    }

    fn get_roll(&self) -> f32 {
        unsafe { crate::natives::entity::get_roll(self.get_handle()) }
    }

    fn get_pitch(&self) -> f32 {
        unsafe { crate::natives::entity::get_pitch(self.get_handle()) }
    }

    fn get_type(&self) -> u32 {
        unsafe { crate::natives::entity::get_type(self.get_handle()) }
    }

    fn get_model(&self) -> Hash {
        unsafe { crate::natives::entity::get_model(self.get_handle()) }
    }

    fn is_animation_finished(&self, dictionary: &str, name: &str) -> bool {
        unsafe {
            crate::natives::entity::is_animation_finished(
                self.get_handle(),
                CString::new(dictionary).unwrap(),
                CString::new(name).unwrap()
            )
        }
    }
}