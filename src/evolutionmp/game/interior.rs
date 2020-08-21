use crate::invoke;
use crate::game::Handle;
use crate::hash::{Hashable, Hash};
use crate::game::entity::Entity;
use cgmath::Vector3;
use crate::native::NativeVector3;
use crate::game::streaming::Resource;

#[derive(Debug)]
pub struct Interior {
    handle: Handle
}

crate::impl_handle!(Interior);

impl Interior {
    pub fn from_gameplay_cam() -> Option<Interior> {
        invoke!(Option<Interior>, 0xE7D267EC6CA966C3)
    }

    pub fn from_entity(entity: &dyn Entity) -> Option<Interior> {
        invoke!(Option<Interior>, 0x2107BA504071A6BB, entity.get_handle())
    }

    pub fn from_collision(pos: Vector3<f32>) -> Option<Interior> {
        invoke!(Option<Interior>, 0xEC4CF9FCB29A4424, pos)
    }

    pub fn from_pos(pos: Vector3<f32>) -> Option<Interior> {
        invoke!(Option<Interior>, 0xB0F7F8663821D9C3, pos)
    }

    pub fn from_pos_and_type<H>(pos: Vector3<f32>, ty: H) -> Option<Interior> where H: Hashable {
        invoke!(Option<Interior>, 0xF0F77ADB9F67E79D, pos, ty.joaat())
    }

    pub fn set_prop_enabled(&self, prop: &str, enabled: bool) {
        if enabled {
            invoke!((), 0x55E86AF2712B36A1, self.handle, prop)
        } else {
            invoke!((), 0x420BD37289EEE162, self.handle, prop)
        }
    }

    pub fn is_prop_enabled(&self, props: &str) -> bool {
        invoke!(bool, 0x35F7DD45E8C0A16D, self.handle, props)
    }

    pub fn set_capped(&self, capped: bool) {
        invoke!((), 0xD9175F941610DB54, self.handle, capped)
    }

    pub fn is_capped(&self) -> bool {
        invoke!(bool, 0x92BAC8ACF88CEC26, self.handle)
    }

    pub fn disable(&self) {
        invoke!((), 0x6170941419D7D8EC, self.handle)
    }

    pub fn is_disabled(&self) -> bool {
        invoke!(bool, 0xBC5115A5A939DD15, self.handle)
    }

    pub fn set_active(&self, active: bool) {
        invoke!((), 0xE37B76C387BE28ED, self.handle, active)
    }

    pub fn is_valid(&self) -> bool {
        invoke!(bool, 0x26B0E73D7EAAF4D3, self.handle)
    }

    pub fn get_group(&self) -> u32 {
        invoke!(u32, 0xE4A84ABF135EF91A, self.handle)
    }

    pub fn get_heading(&self) -> f32 {
        invoke!(f32, 0xF49B58631D9E22D9, self.handle)
    }

    pub fn get_info(&self) -> InteriorInfo {
        let mut pos = NativeVector3::zero();
        let mut hash = Hash(0);
        invoke!((), 0x252BDC06B73FA6EA, self.handle, &mut pos, &mut hash);
        InteriorInfo { pos: pos.into(), hash }
    }

    pub fn get_offset_from(&self, pos: Vector3<f32>) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x9E3B3E6D66F6E22F, self.handle, pos)
    }

    pub fn refresh(&self) {
        invoke!((), 0x41F37C3427C75AE0, self.handle)
    }
}

impl Resource for Interior {
    fn is_loaded(&self) -> bool {
        invoke!(bool, 0x6726BDCCC1932F0E, self.handle)
    }

    fn request(&self) {
        invoke!((), 0x2CA429C029CCF247, self.handle)
    }

    fn mark_unused(&mut self) {
        invoke!((), 0x261CCE7EED010641, self.handle)
    }
}

pub struct InteriorInfo {
    pub pos: Vector3<f32>,
    pub hash: Hash
}