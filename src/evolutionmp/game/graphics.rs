use crate::invoke;
use cgmath::Vector3;
use crate::game::Rgba;
use crate::game::streaming::Texture;

pub fn draw_marker(ty: u32, pos: Vector3<f32>, dir: Vector3<f32>, rot: Vector3<f32>,
                   scale: Vector3<f32>, color: Rgba, bobbing: bool, face_camera: bool, rotate: bool,
                   texture: Option<Texture>, draw_on_entities: bool) {
    invoke!((), 0x28477EC23D892089, ty, pos, dir, rot, scale, color, bobbing, face_camera, 2u32, rotate, texture, draw_on_entities)
}

pub fn draw_line(start: Vector3<f32>, end: Vector3<f32>, color: Rgba) {
    invoke!((), 0x6B7256074AE34680, start, end, color)
}