use crate::invoke;
use crate::game::entity::Entity;
use crate::game::Handle;
use crate::native::pool::Handleable;
use cgmath::Vector3;

pub struct Probe {
    handle: Handle
}

impl Probe {
    pub fn new_ray(start: Vector3<f32>, end: Vector3<f32>, flags: u32, entity: &dyn Entity, p8: u32) -> Probe {
        invoke!(Probe, 0x377906D8A31E5586, start, end, flags, entity.get_handle(), p8)
    }

    pub fn get_result(&self) -> ProbeResult {
        let mut hit = false;
        let mut end = Vector3::new(0.0, 0.0, 0.0);
        let mut surface_normal = Vector3::new(0.0, 0.0, 0.0);
        let mut entity = 0 as Handle;
        let code = invoke!(u32, 0x3D87450E15D98694, self.handle, &mut hit, &mut end, &mut surface_normal, &mut entity);
        ProbeResult {
            hit,
            end,
            surface_normal,
            entity: if entity == 0 { None } else { Some(ProbeEntity { handle: entity }) },
            code
        }
    }
}

impl Handleable for Probe {
    fn from_handle(handle: Handle) -> Option<Self> where Self: Sized {
        if handle == 0 {
            None
        } else {
            Some(Probe { handle })
        }
    }

    fn get_handle(&self) -> Handle {
        self.handle
    }
}

pub struct ProbeResult {
    pub hit: bool,
    pub end: Vector3<f32>,
    pub surface_normal: Vector3<f32>,
    pub entity: Option<ProbeEntity>,
    pub code: u32
}

pub struct ProbeEntity {
    handle: Handle
}

impl Entity for ProbeEntity {
    fn delete(&mut self) {
        self.set_persistent(false);
        invoke!((), 0xAE3CBE5BF394C9C9, &mut self.handle)
    }
}

impl Handleable for ProbeEntity {
    fn from_handle(handle: u32) -> Option<Self> where Self: Sized {
        if handle == 0 {
            None
        } else {
            Some(ProbeEntity { handle })
        }
    }

    fn get_handle(&self) -> u32 {
        self.handle
    }
}