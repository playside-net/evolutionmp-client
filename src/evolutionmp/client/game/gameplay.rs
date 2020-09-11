use crate::invoke;

pub fn lower_map_prop_density(lower: bool) {
    invoke!((), 0x9BAE5AD2508DF078, lower)
}

pub fn set_time_scale(scale: f32) {
    invoke!((), 0x1D408577D440E81E, scale)
}