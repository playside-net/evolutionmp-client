use crate::invoke;
use crate::game::entity::Entity;
use crate::game::Handle;
use crate::native::pool::Handleable;
use crate::hash::Hash;
use crate::native::NativeVector3;
use cgmath::{Vector3, Array};

#[derive(Debug)]
pub struct Probe {
    handle: Handle
}

impl Probe {
    pub fn new_ray(start: Vector3<f32>, end: Vector3<f32>, flags: i32, entity: &dyn Entity, p8: u32) -> Probe {
        invoke!(Probe, 0x377906D8A31E5586, start, end, flags, entity.get_handle(), p8)
    }

    pub fn get_result(&self, include_material: bool) -> ProbeResult {
        let mut hit = false;
        let mut end = NativeVector3::zero();
        let mut surface_normal = NativeVector3::zero();
        let mut entity = 0 as Handle;
        let mut material = Hash(0);
        let code = if include_material {
            invoke!(u32, 0x65287525D951F6BE, self.handle, &mut hit, &mut end, &mut surface_normal, &mut material, &mut entity)
        } else {
            invoke!(u32, 0x3D87450E15D98694, self.handle, &mut hit, &mut end, &mut surface_normal, &mut entity)
        };
        ProbeResult {
            hit,
            end: end.into(),
            surface_normal: surface_normal.into(),
            entity: if entity == 0 { None } else { Some(ProbeEntity { handle: entity }) },
            material: if include_material { Some(material) } else { None },
            code
        }
    }
}

crate::impl_handle!(Probe);

#[derive(Debug)]
pub struct ProbeResult {
    pub hit: bool,
    pub end: Vector3<f32>,
    pub surface_normal: Vector3<f32>,
    pub entity: Option<ProbeEntity>,
    pub material: Option<Hash>,
    pub code: u32
}

#[derive(Debug)]
pub struct ProbeEntity {
    handle: Handle
}

impl Entity for ProbeEntity {
    fn delete(&mut self) {
        self.set_persistent(false);
        invoke!((), 0xAE3CBE5BF394C9C9, &mut self.handle)
    }
}

crate::impl_handle!(ProbeEntity);