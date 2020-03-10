use crate::invoke;
use crate::game::streaming::Texture;
use crate::game::Rgba;
use cgmath::Vector3;

pub fn draw_marker(ty: u32, pos: Vector3<f32>, dir: Vector3<f32>, rot: Vector3<f32>,
                   scale: Vector3<f32>, color: Rgba, bobbing: bool, face_camera: bool, rotate: bool,
                   texture: Option<Texture>, draw_on_entities: bool) {
    invoke!((), 0x28477EC23D892089, ty, pos, dir, rot, scale, color, bobbing, face_camera, 2u32, rotate, texture, draw_on_entities)
}

pub fn draw_line(start: Vector3<f32>, end: Vector3<f32>, color: Rgba) {
    invoke!((), 0x6B7256074AE34680, start, end, color)
}

pub fn set_artificial_light(enabled: bool) {
    invoke!((), 0x1268615ACE24D504, enabled)
}

pub mod night_vision {
    use crate::invoke;

    pub fn set_enabled(enabled: bool) {
        invoke!((), 0x18F621F7A5B1F85D, enabled)
    }

    pub fn is_enabled() -> bool {
        invoke!(bool, 0x2202A3F42C8E5F79)
    }
}

pub mod heat_vision {
    use crate::invoke;
    use crate::game::Rgb;

    pub fn is_enabled() -> bool {
        invoke!(bool, 0x44B80ABAB9D80BD3)
    }

    pub fn get_max_thickness() -> f32 {
        invoke!(f32, 0x43DBAE39626CE83F)
    }

    pub fn reset() {
        invoke!((), 0x70A64C0234EF522C)
    }

    pub fn set_color_near(color: Rgb) {
        invoke!((), 0x1086127B3A63505E, color)
    }

    pub fn set_fade_start_distance(distance: f32) {
        invoke!((), 0xA78DE25577300BA1, distance)
    }

    pub fn set_fade_end_distance(distance: f32) {
        invoke!((), 0x9D75795B9DC6EBBF, distance)
    }

    pub fn set_heat_scale(scale: f32) {
        invoke!((), 0xD7D0B00177485411, scale)
    }

    pub fn set_hi_light_intensity(intensity: f32) {
        invoke!((), 0x19E50EB6E33E1D28, intensity)
    }

    pub fn set_hi_light_noise(noise: f32) {
        invoke!((), 0x1636D7FC127B10D2, noise)
    }

    pub fn set_max_thickness(thickness: f32) {
        invoke!((), 0x0C8FAC83902A62DF, thickness)
    }

    pub fn set_min_noise_amount(noise: f32) {
        invoke!((), 0xFF5992E1C9E65D05, noise)
    }

    pub fn set_max_noise_amount(noise: f32) {
        invoke!((), 0xFEBFBFDFB66039DE, noise)
    }
}

pub mod timecycle {
    use crate::invoke;

    pub fn clear_secondary_modifier() {
        invoke!((), 0x92CCC17A7A2285DA)
    }

    pub fn clear_primary_modifier() {
        invoke!((), 0x0F07E7745A236711)
    }

    pub fn get_secondary_modifier_index() -> i32 {
        invoke!(i32, 0xBB0527EC6341496D)
    }

    pub fn get_primary_modifier_index() -> i32 {
        invoke!(i32, 0xFDF3D97C674AFB66)
    }

    pub fn get_transition_modifier_index() -> i32 {
        invoke!(i32, 0x459FD2C8D0AB78BC)
    }

    pub fn pop_primary_modifier() {
        invoke!((), 0x3C8938D7D872211E)
    }

    pub fn push_primary_modifier() {
        invoke!((), 0x58F735290861E6B4)
    }

    pub fn reset_secondary_modifier_strength() {
        invoke!((), 0x2BF72AD5B41AA739)
    }

    pub fn set_secondary_modifier(modifier: f32) {
        invoke!((), 0x5096FD9CCB49056D, modifier)
    }

    pub fn set_primary_modifier_strength(strength: f32) {
        invoke!((), 0x82E7FFCD5B2326B3, strength)
    }

    pub fn set_secondary_modifier_strength(strength: f32) {
        invoke!((), 0x2C328AF17210F009, strength)
    }

    pub fn set_next_player_modifier(modifier: &str) {
        invoke!((), 0xBF59707B3E5ED531, modifier)
    }

    pub fn set_primary_modifier(modifier: &str) {
        invoke!((), 0x2C933ABF17A1DF41, modifier)
    }

    pub fn set_transition_modifier(modifier: &str, transition: f32) {
        invoke!((), 0x3BCF567485E1971C, modifier)
    }
}