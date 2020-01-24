use crate::invoke;
use crate::game::{Handle, Rgba};
use cgmath::{Vector2, Vector3};
use crate::native::pool::Handleable;

pub struct Scaleform {
    handle: Handle,
    color: Rgba
}

impl Scaleform {
    pub fn new(id: &str, color: Rgba) -> Option<Scaleform> {
        let handle = invoke!(Handle, 0x11FE353CF9733E6F, id);
        if handle > 0 {
            Some(Scaleform {
                handle, color
            })
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.handle > 0
    }

    pub fn is_loaded(&self) -> bool {
        invoke!(bool, 0x85F01B8D5B90570E, self.handle)
    }

    pub fn invoke<R>(&self, method: &str, args: &[ScaleformArg]) -> R where R: ScaleformResult {
        invoke!((), 0xF6E48914C7A8694E, self.handle, method);
        for arg in args {
            match arg {
                ScaleformArg::I32(i) => invoke!((), 0xC3D0841A0CC546A6, *i),
                ScaleformArg::F32(f) => invoke!((), 0xD69736AAE04DB51A, *f),
                ScaleformArg::Bool(b) => invoke!((), 0xC58424BA936EB458, *b),
                ScaleformArg::Str(s) => invoke!((), 0xBA7148484BD90365, s.as_str())
            }
        }
        R::read(self.handle)
    }

    pub fn render(&self, pos: Vector2<f32>, size: Vector2<f32>) {
        invoke!((), 0x54972ADAF0294A93, self.handle, pos, size, self.color, 0u32)
    }

    pub fn render_fullscreen(&self) {
        invoke!((), 0x54972ADAF0294A93, self.handle, self.color, 0u32)
    }

    pub fn render_volumetric(&self, pos: Vector3<f32>, rot: Vector3<f32>, scale: Vector3<f32>, additive: bool) {
        if additive {
            invoke!((), 0x87D51D72255D4E78, self.handle, pos, rot, 2.0, 2.0, 1.0, scale, 2)
        } else {
            invoke!((), 0x1CE592FDC749D6F5, self.handle, pos, rot, 2.0, 2.0, 1.0, scale, 2)
        }
    }
}

impl std::ops::Drop for Scaleform {
    fn drop(&mut self) {
        invoke!((), 0x1D132D614DD86811, &mut self.handle);
    }
}

pub enum ScaleformArg {
    I32(i32), F32(f32), Bool(bool), Str(String)
}

pub trait ScaleformResult {
    fn read(handle: Handle) -> Self where Self: Sized;
}

impl ScaleformResult for () {
    fn read(handle: Handle) -> Self where Self: Sized {
        invoke!((), 0xC6796A8FFA375E53)
    }
}

impl ScaleformResult for i32 {
    fn read(handle: Handle) -> Self where Self: Sized {
        let ret = end_method_returnable();
        invoke!(i32, 0x2DE7EFA66B906036, ret)
    }
}

impl ScaleformResult for bool {
    fn read(handle: Handle) -> Self where Self: Sized {
        let ret = end_method_returnable();
        invoke!(bool, 0x768FF8961BA904D6, ret)
    }
}

fn end_method_returnable() -> Handle {
    invoke!(Handle, 0xC50AA39A577AF886)
}

impl Handleable for Scaleform {
    fn from_handle(handle: u32) -> Option<Self> where Self: Sized {
        unimplemented!()
    }

    fn get_handle(&self) -> u32 {
        unimplemented!()
    }
}