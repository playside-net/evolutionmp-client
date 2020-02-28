use crate::{invoke,invoke_option, impl_handle};
use crate::game::{Handle, Rgb};
use crate::runtime::ScriptEnv;
use crate::game::entity::Entity;
use crate::hash::Hashable;
use crate::game::streaming::Model;
use crate::native::pool::GenericPool;
use cgmath::Vector3;
use std::mem::ManuallyDrop;

pub fn get_pool() -> ManuallyDrop<Box<GenericPool<Prop>>> {
    crate::native::pool::get_props().expect("prop pool not initialized")
}

pub struct Prop {
    handle: Handle
}

impl Prop {
    pub fn new<H>(env: &mut ScriptEnv, model: H, pos: Vector3<f32>, is_network: bool, this_script_check: bool, dynamic: bool) -> Option<Prop> where H: Hashable {
        let model = Model::from(model);
        if model.is_in_cd_image() && model.is_valid() {
            env.wait_for_resource(&model);
            invoke!(Option<Prop>, 0x509D5878EB39E842, model.joaat(), pos, is_network, this_script_check, dynamic)
        } else {
            None
        }
    }

    pub fn find_nearest<H>(pos: Vector3<f32>, radius: f32, model: H) -> Option<Prop> where H: Hashable {
        invoke!(Option<Prop>, 0xE143FA2249364369, pos, radius, model.joaat(), false, false, false)
    }

    pub fn get_texture_variation(&self) -> i32 {
        invoke!(i32, 0xE84EB93729C5F36A, self.handle)
    }

    pub fn set_climbable(&self, climbable: bool) {
        invoke!((), 0x4D89D607CB3DD1D2, self.handle, climbable)
    }

    pub fn set_color(&self, toggle: bool, color: Rgb) {
        invoke!((), 0x3B2FD68DB5F8331C, self.handle, toggle, color)
    }

    pub fn set_light_color(&self, toggle: bool, color: Rgb) {
        invoke!((), 0x5F048334B4A4E774, self.handle, toggle, color)
    }

    pub fn set_targetable(&self, targetable: bool) {
        invoke!((), 0x8A7391690F5AFD81, self.handle, targetable)
    }

    pub fn set_paint(&self, paint: u32) {
        invoke!((), 0x971DA0055324D033, self.handle, paint)
    }

    pub fn is_broken(&self) -> bool {
        invoke!(bool, 0x8ABFB70C49CC43E2, self.handle)
    }

    pub fn is_visible(&self) -> bool {
        invoke!(bool, 0x8B32ACE6326A7546, self.handle)
    }

    pub fn mark_unused(&self) {
        invoke!((), 0xADBE4809F19F927A, self.handle)
    }

    pub fn place_on_ground_properly(&self) {
        invoke!((), 0x58A850EAEE20FAA3, self.handle)
    }

    pub fn slide_to(&self, pos: Vector3<f32>, velocity: Vector3<f32>, collide: bool) {
        invoke!((), 0x2FDFF4107B8C1147, self.handle, pos, velocity, collide)
    }
}

impl Entity for Prop {
    fn delete(&mut self) {
        invoke!((), 0x539E0AE3E6634B9F, &mut self.handle)
    }
}

impl_handle!(Prop);