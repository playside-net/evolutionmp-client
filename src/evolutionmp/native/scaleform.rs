use crate::invoke;
use crate::game::{Handle, Vector2, Rgba, Vector3};

pub unsafe fn request(id: &str) -> Handle {
    invoke!(Handle, 0x11FE353CF9733E6F, id)
}

pub unsafe fn has_loaded(handle: Handle) -> bool {
    invoke!(bool, 0x85F01B8D5B90570E, handle)
}

pub unsafe fn drop(handle: &mut Handle) {
    invoke!((), 0x1D132D614DD86811, handle)
}

pub unsafe fn begin_method(handle: Handle, name: &str) {
    invoke!((), 0xF6E48914C7A8694E, handle, name)
}

pub unsafe fn push_i32(value: i32) {
    invoke!((), 0xC3D0841A0CC546A6, value)
}

pub unsafe fn push_f32(value: f32) {
    invoke!((), 0xD69736AAE04DB51A, value)
}

pub unsafe fn push_bool(value: bool) {
    invoke!((), 0xC58424BA936EB458, value)
}

pub unsafe fn push_str(value: &str) {
    invoke!((), 0xBA7148484BD90365, value)
}

pub unsafe fn end_method() {
    invoke!((), 0xC6796A8FFA375E53)
}

pub unsafe fn end_method_returnable() -> Handle {
    invoke!(Handle, 0xC50AA39A577AF886)
}

pub unsafe fn get_method_return_value_bool(handle: Handle) -> bool {
    invoke!(bool, 0x768FF8961BA904D6, handle)
}

pub unsafe fn get_method_return_value_int(handle: Handle) -> i32 {
    invoke!(i32, 0x2DE7EFA66B906036, handle)
}

pub unsafe fn render(handle: Handle, pos: Vector2<f32>, size: Vector2<f32>, color: Rgba, unk: i32) {
    invoke!((), 0x54972ADAF0294A93, handle, pos, size, color, unk)
}

pub unsafe fn render_fullscreen(handle: Handle, color: Rgba, unk: i32) {
    invoke!((), 0x54972ADAF0294A93, handle, color, unk)
}

pub unsafe fn render_volumetric(handle: Handle, pos: Vector3<f32>, rot: Vector3<f32>, p4: f32, sharpness: f32, p6: f32, scale: Vector3<f32>, p8: i32) {
    invoke!((), 0x87D51D72255D4E78, handle, pos, rot, p4, sharpness, p6, scale, p8)
}

pub unsafe fn render_volumetric_non_additive(handle: Handle, pos: Vector3<f32>, rot: Vector3<f32>, p4: f32, sharpness: f32, p6: f32, scale: Vector3<f32>, p8: i32) {
    invoke!((), 0x1CE592FDC749D6F5, handle, pos, rot, p4, sharpness, p6, scale, p8)
}