use crate::game::{Handle, Vector3, Vector2, Rgba};

pub struct Scaleform {
    handle: Handle,
    color: Rgba
}

impl Scaleform {
    pub fn new(id: &str, color: Rgba) -> Option<Scaleform> {
        let handle = unsafe { crate::native::scaleform::request(id) };
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
        unsafe {
            crate::native::scaleform::has_loaded(self.handle)
        }
    }

    pub fn invoke<R>(&self, method: &str, args: &[ScaleformArg]) -> R where R: ScaleformResult {
        unsafe {
            crate::native::scaleform::begin_method(self.handle, method);
            for arg in args {
                match arg {
                    ScaleformArg::I32(i) => crate::native::scaleform::push_i32(*i),
                    ScaleformArg::F32(f) => crate::native::scaleform::push_f32(*f),
                    ScaleformArg::Bool(b) => crate::native::scaleform::push_bool(*b),
                    ScaleformArg::Str(s) => crate::native::scaleform::push_str(s.as_str())
                }
            }
            R::read(self.handle)
        }
    }

    pub fn render(&self, pos: Vector2, size: Vector2) {
        unsafe {
            crate::native::scaleform::render(self.handle, pos, size, self.color, 0)
        }
    }

    pub fn render_fullscreen(&self) {
        unsafe {
            crate::native::scaleform::render_fullscreen(self.handle, self.color, 0)
        }
    }

    pub fn render_volumetric(&self, pos: Vector3, rot: Vector3, scale: Vector3, additive: bool) {
        unsafe {
            if additive {
                crate::native::scaleform::render_volumetric(self.handle, pos, rot, 2.0, 2.0, 1.0, scale, 2)
            } else {
                crate::native::scaleform::render_volumetric_non_additive(self.handle, pos, rot, 2.0, 2.0, 1.0, scale, 2)
            }
        }
    }
}

impl std::ops::Drop for Scaleform {
    fn drop(&mut self) {
        unsafe {
            crate::native::scaleform::drop(&mut self.handle);
        }
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
        unsafe {
            crate::native::scaleform::end_method()
        }
    }
}

impl ScaleformResult for i32 {
    fn read(handle: Handle) -> Self where Self: Sized {
        unsafe {
            let ret = crate::native::scaleform::end_method_returnable();
            crate::native::scaleform::get_method_return_value_int(ret)
        }
    }
}

impl ScaleformResult for bool {
    fn read(handle: Handle) -> Self where Self: Sized {
        unsafe {
            let ret = crate::native::scaleform::end_method_returnable();
            crate::native::scaleform::get_method_return_value_bool(ret)
        }
    }
}