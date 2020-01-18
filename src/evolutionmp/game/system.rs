use crate::invoke;
use cgmath::Vector3;

pub fn get_timer_a() -> u32 {
    invoke!(u32, 0x83666F9FB8FEBD4B)
}

pub fn get_timer_b() -> u32 {
    invoke!(u32, 0xC9D9444186B5A374)
}

pub fn set_timer_a(timer: u32) {
    invoke!((), 0xC1B1E9A034A63A62, timer)
}

pub fn set_timer_b(timer: u32) {
    invoke!((), 0x5AE11BC36633DE4E, timer)
}

pub fn get_time_step() -> f32 {
    invoke!(f32, 0x0000000050597EE2)
}

pub fn cos(value: f32) -> f32 {
    invoke!(f32, 0xD0FFB162F40A139C, value)
}

pub fn sin(value: f32) -> f32 {
    invoke!(f32, 0x0BADBFA3B172435F, value)
}

pub fn sqrt(value: f32) -> f32 {
    invoke!(f32, 0x71D93B57D07F9804, value)
}

pub fn pow(base: f32, exponent: f32) -> f32 {
    invoke!(f32, 0xE3621CC40F31FE2E, base, exponent)
}

pub fn vector_magnitude(v: Vector3<f32>) -> f32 {
    invoke!(f32, 0x652D2EEEF1D3E62C, v)
}

pub fn vector_magnitude2(v: Vector3<f32>) -> f32 {
    invoke!(f32, 0xA8CEACB4F35AE058, v)
}

pub fn vector_distance(start: Vector3<f32>, end: Vector3<f32>) -> f32 {
    invoke!(f32, 0x2A488C176D52CCA5, start, end)
}

pub fn vector_distance2(start: Vector3<f32>, end: Vector3<f32>) -> f32 {
    invoke!(f32, 0xB7A628320EFF8E47, start, end)
}

pub fn shift_left(value: u32, bits: u32) -> u32 {
    invoke!(u32, 0xEDD95A39E5544DE8, value, bits)
}

pub fn floor(value: f32) -> u32 {
    invoke!(u32, 0xF34EE736CF047844, value)
}

pub fn ceil(value: f32) -> u32 {
    invoke!(u32, 0x11E019C8F43ACC8A, value)
}

pub fn round(value: f32) -> u32 {
    invoke!(u32, 0xF2DB717A73826179, value)
}