use crate::hash::Hash;
use super::Handle;
use crate::native;
use crate::native::pool::FromHandle;
use cgmath::Vector3;

pub trait Entity {
    fn get_handle(&self) -> Handle;

    fn exists(&self) -> bool {
        unsafe { native::entity::exists(self.get_handle()) }
    }

    fn get_address(&self) -> *mut u8 {
        unsafe { (native::pool::ENTITY_ADDRESS.unwrap())(self.get_handle()) }
    }

    fn is_dead(&self) -> bool {
        unsafe { native::entity::is_dead(self.get_handle()) }
    }

    fn get_position(&self) -> Vector3<f32> {
        unsafe { native::entity::get_position(self.get_handle()) }
    }

    fn set_position_no_offset(&self, pos: Vector3<f32>, axis: Vector3<bool>) {
        unsafe { native::entity::set_position_no_offset(self.get_handle(), pos, axis) }
    }

    fn get_rotation(&self, order: u32) -> Vector3<f32> {
        unsafe { native::entity::get_rotation(self.get_handle(), order) }
    }

    fn get_rotation_velocity(&self) -> Vector3<f32> {
        unsafe { native::entity::get_rotation_velocity(self.get_handle()) }
    }

    fn get_heading(&self) -> f32 {
        unsafe { native::entity::get_heading(self.get_handle()) }
    }

    fn get_roll(&self) -> f32 {
        unsafe { native::entity::get_roll(self.get_handle()) }
    }

    fn get_pitch(&self) -> f32 {
        unsafe { native::entity::get_pitch(self.get_handle()) }
    }

    fn get_type(&self) -> u32 {
        unsafe { native::entity::get_type(self.get_handle()) }
    }

    fn get_model(&self) -> Hash {
        unsafe { native::entity::get_model(self.get_handle()) }
    }

    fn is_animation_finished(&self, dictionary: &str, name: &str) -> bool {
        unsafe {
            native::entity::is_animation_finished(
                self.get_handle(),
                dictionary,
                name
            )
        }
    }

    fn set_position_freezed(&self, freezed: bool) {
        unsafe { native::entity::set_position_freezed(self.get_handle(), freezed) }
    }

    fn set_dynamic(&self, dynamic: bool) {
        unsafe { native::entity::set_dynamic(self.get_handle(), dynamic) }
    }

    fn set_collision(&self, collision: bool, physics: bool) {
        unsafe { native::entity::set_collision(self.get_handle(), collision, physics) }
    }

    fn get_position_by_offset(&self, offset: Vector3<f32>) -> Vector3<f32> {
        unsafe { native::entity::get_position_by_offset(self.get_handle(), offset) }
    }

    fn delete(&mut self);

    fn set_persistent(&self, persistent: bool) {
        unsafe { native::entity::set_as_mission(self.get_handle(), persistent, !persistent) }
    }
}