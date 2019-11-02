use crate::invoke;

pub unsafe fn get_timer_a() -> u32 {
    invoke!(u32, 0x83666F9FB8FEBD4B)
}

pub unsafe fn get_timer_b() -> u32 {
    invoke!(u32, 0xC9D9444186B5A374)
}

pub unsafe fn set_timer_a(timer: u32) {
    invoke!((), 0xC1B1E9A034A63A62, timer)
}

pub unsafe fn set_timer_b(timer: u32) {
    invoke!((), 0x5AE11BC36633DE4E, timer)
}

pub unsafe fn get_time_step() -> f32 {
    invoke!(f32, 0x0000000050597EE2)
}

pub unsafe fn cos(value: f32) -> f32 {
    invoke!(f32, 0xD0FFB162F40A139C, value)
}

pub unsafe fn sin(value: f32) -> f32 {
    invoke!(f32, 0x0BADBFA3B172435F, value)
}

pub unsafe fn sqrt(value: f32) -> f32 {
    invoke!(f32, 0x71D93B57D07F9804, value)
}

pub unsafe fn pow(base: f32, exponent: f32) -> f32 {
    invoke!(f32, 0xE3621CC40F31FE2E, base, exponent)
}

pub unsafe fn vector_magnitude(x: f32, y: f32, z: f32) -> f32 {
    invoke!(f32, 0x652D2EEEF1D3E62C, x, y, z)
}

pub unsafe fn vector_magnitude2(x: f32, y: f32, z: f32) -> f32 {
    invoke!(f32, 0xA8CEACB4F35AE058, x, y, z)
}

pub unsafe fn vector_distance(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> f32 {
    invoke!(f32, 0x2A488C176D52CCA5, x1, y1, z1, x2, y2, z2)
}

pub unsafe fn vector_distance2(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> f32 {
    invoke!(f32, 0xB7A628320EFF8E47, x1, y1, z1, x2, y2, z2)
}

pub unsafe fn shift_left(value: u32, bits: u32) -> u32 {
    invoke!(u32, 0xEDD95A39E5544DE8, value, bits)
}

pub unsafe fn floor(value: f32) -> u32 {
    invoke!(u32, 0xF34EE736CF047844, value)
}

pub unsafe fn ceil(value: f32) -> u32 {
    invoke!(u32, 0x11E019C8F43ACC8A, value)
}

pub unsafe fn round(value: f32) -> u32 {
    invoke!(u32, 0xF2DB717A73826179, value)
}