use crate::hash::Hash;
use super::Handle;
use crate::invoke;
use crate::native::pool::{self, Handleable};
use crate::native::{NativeField, Addressable};
use crate::game::streaming::{Resource, AnimDict};
use cgmath::{Vector3, Euler, Deg, Quaternion, Matrix3};

pub trait Entity: Handleable {
    fn exists(&self) -> bool {
        invoke!(bool, 0x7239B21A38F536BA, self.get_handle())
    }

    fn is_dead(&self) -> bool {
        invoke!(bool, 0x5F9532F3B5CC2551, self.get_handle())
    }

    fn get_position(&self) -> Vector3<f32> {
        let alive = !self.is_dead();
        invoke!(Vector3<f32>, 0x3FEF770D40960D5A, self.get_handle(), alive)
    }

    fn set_position(&self, pos: Vector3<f32>, invert_axis: Vector3<bool>, clear_area: bool) {
        invoke!((), 0x06843DA7060A026B, self.get_handle(), pos, invert_axis, clear_area)
    }

    fn set_position_no_offset(&self, pos: Vector3<f32>, invert_axis: Vector3<bool>) {
        invoke!((), 0x239A3351AC1DA385, self.get_handle(), pos, invert_axis)
    }

    fn set_load_collision(&self, load: bool) {
        invoke!((), 0x0DC7CABAB1E9B67E, self.get_handle(), load)
    }

    fn get_rotation(&self, order: u32) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0xAFBD61CC738D9EB9, self.get_handle(), order as u32)
    }

    fn set_rotation(&self, rotation: Vector3<f32>, order: u32) {
        invoke!((), 0x8524A8B0171D5E07, self.get_handle(), rotation, order as u32, true)
    }

    fn get_rotation_velocity(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x213B91045D09B983, self.get_handle())
    }

    fn get_velocity(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x4805D2B1D8CF94A9, self.get_handle())
    }

    fn set_velocity(&self, velocity: Vector3<f32>) {
        invoke!((), 0x1C99BB7B6E96D16F, self.get_handle(), velocity)
    }

    fn get_heading(&self) -> f32 {
        invoke!(f32, 0xE83D4F9BA2A38914, self.get_handle())
    }

    fn set_heading(&self, heading: f32) {
        invoke!((), 0x8E2530AA8ADA980E, self.get_handle(), heading)
    }

    fn get_roll(&self) -> f32 {
        invoke!(f32, 0x831E0242595560DF, self.get_handle())
    }

    fn get_pitch(&self) -> f32 {
        invoke!(f32, 0xD45DC2893621E1FE, self.get_handle())
    }

    fn get_type(&self) -> u32 {
        invoke!(u32, 0x8ACD366038D14505, self.get_handle())
    }

    fn get_model(&self) -> Hash {
        invoke!(Hash, 0x9F47B058362C84B5, self.get_handle())
    }

    fn is_animation_finished(&self, dictionary: &str, name: &str) -> bool {
        invoke!(bool, 0x20B711662962B472, self.get_handle(), dictionary, name)
    }

    fn set_invincible(&self, invincible: bool) {
        invoke!((), 0x3882114BDE571AD4, self.get_handle(), invincible)
    }

    fn is_visible(&self) -> bool {
        invoke!(bool, 0x47D6F43D77935C75, self.get_handle())
    }

    fn set_visible(&self, visible: bool) {
        invoke!((), 0xEA1C610A04DB6BBB, self.get_handle(), visible, false)
    }

    fn set_position_freezed(&self, freezed: bool) {
        invoke!((), 0x428CA6DBD1094446, self.get_handle(), freezed)
    }

    fn set_dynamic(&self, dynamic: bool) {
        invoke!((), 0x1718DE8E3F2823CA, self.get_handle(), dynamic)
    }

    fn set_collision(&self, collision: bool, physics: bool) {
        invoke!((), 0x1A9205C1B9EE827F, self.get_handle(), collision, physics)
    }

    fn get_position_by_offset(&self, offset: Vector3<f32>) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x1899F328B0E12848, self.get_handle(), offset)
    }

    fn delete(&mut self);

    fn set_persistent(&self, persistent: bool) {
        invoke!((), 0xAD738C3085FE7E11, self.get_handle(), persistent, !persistent)
    }

    fn is_belong_to_this_script(&self, p2: bool) -> bool {
        invoke!(bool, 0xDDE6DF5AE89981D2, self.get_handle(), p2)
    }

    fn has_drawable(&self) -> bool {
        invoke!(bool, 0x060D6E96F8B8E48D, self.get_handle())
    }

    fn has_physics(&self) -> bool {
        invoke!(bool, 0xDA95EA3317CC5064, self.get_handle())
    }

    fn is_entity(&self) -> bool {
        invoke!(bool, 0x731EC8A916BD11A1, self.get_handle())
    }

    fn get_health(&self) -> u32 {
        invoke!(u32, 0xEEF059FAD016D209, self.get_handle())
    }

    fn get_max_health(&self) -> u32 {
        invoke!(u32, 0x15D757606D170C3C, self.get_handle())
    }

    fn set_health(&self, health: u32) {
        invoke!((), 0x6B76DC1F3AE6E6A3, self.get_handle(), health)
    }

    fn set_max_health(&self, health: u32) {
        invoke!((), 0x166E7CF68597D8B5, self.get_handle(), health)
    }

    fn is_animation_playing(&self, dict: &AnimDict, name: &str, flag: u32) -> bool {
        invoke!(bool, 0x1F0B79228E461EC9, self.get_handle(), dict.get_name(), name, flag)
    }

    fn is_in_water(&self) -> bool {
        invoke!(bool, 0xCFB0A0D8EDD145A3, self.get_handle())
    }
}

impl<E> Addressable for E where E: Entity + Sized {
    fn get_address(&self) -> *mut u8 {
        (pool::ENTITY_ADDRESS.get().unwrap())(self.get_handle())
    }
}

pub struct Bone<'a, E> where E: Entity {
    pub entity: &'a E,
    pub index: i32
}

impl<'a, E> Bone<'a, E> where E: Entity {
    pub fn get_entity(&self) -> &'a E {
        self.entity
    }

    pub fn attach(&self, entity: &dyn Entity, pos: Vector3<f32>, rotation: Vector3<f32>) {
        invoke!((), 0x6B9BBD38AB0796DF, entity.get_handle(), self.entity.get_handle(), self.index, pos, rotation, false, false, false, false, 2, true)
    }

    pub fn get_position(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x44A8FCB8ED227738, self.entity.get_handle(), self.index)
    }

    pub fn get_rotation(&self) -> Quaternion<f32> {
        invoke!(Quaternion<f32>, 0xCE6294A232D03786, self.entity.get_handle(), self.index)
    }
}