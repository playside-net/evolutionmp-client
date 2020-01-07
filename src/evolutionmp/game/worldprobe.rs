use crate::native;
use cgmath::Vector3;
use crate::game::entity::Entity;
use crate::game::Handle;
use crate::native::scaleform::end_method_returnable;
use crate::native::pool::FromHandle;

pub struct Probe {
    handle: Handle
}

impl Probe {
    pub fn new_ray(start: Vector3<f32>, end: Vector3<f32>, flags: u32, entity: &dyn Entity, p8: u32) -> Probe {
        Probe {
            handle: unsafe { native::worldprobe::new_ray(start, end, flags, entity.get_handle(), p8) }
        }
    }

    pub fn get_result(&self) -> ProbeResult {
        let mut hit = false;
        let mut end = Vector3::new(0.0, 0.0, 0.0);
        let mut surface_normal = Vector3::new(0.0, 0.0, 0.0);
        let mut entity = 0 as Handle;
        let code = unsafe {
            native::worldprobe::get_result(self.handle, &mut hit, &mut end, &mut surface_normal, &mut entity)
        };
        ProbeResult {
            hit,
            end,
            surface_normal,
            entity: if entity == 0 { None } else { Some(ProbeEntity { handle: entity }) },
            code
        }
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
    fn get_handle(&self) -> u32 {
        self.handle
    }

    fn delete(&mut self) {
        self.set_persistent(false);
        unsafe { native::entity::delete(&mut self.handle) }
    }
}