use crate::invoke;
use cgmath::{Vector3, Vector2};
use crate::native::NativeVector3;

pub fn set_active(active: bool) {
    invoke!((), 0x3BD3F52BA9B1E4E8, active)
}

pub fn start_multi_route(color: u32, from_player: bool, display_outside_vehicle: bool, points: &[Vector3<f32>]) {
    clear_multi_route();

    invoke!((), 0x3D3D15AF7BCAAF83, color, from_player, display_outside_vehicle);

    for point in points {
        add_multi_route_point(*point);
    }
    set_multi_route_visible(true);
}

pub fn clear_multi_route() {
    invoke!((), 0x67EEDEA1B9BAFD94)
}

fn set_multi_route_visible(visible: bool) {
    invoke!((), 0x3DDA37128DD1ACA8, visible)
}

fn add_multi_route_point(pos: Vector3<f32>) {
    invoke!((), 0xA905192A6781C41B, pos)
}

pub fn start_custom_route(color: u32, display_outside_vehicle: bool, follow_player: bool,
                          radar_thickness: u32, map_thickness: u32, points: &[Vector3<f32>]) {
    clear_custom_route();

    invoke!((), 0xDB34E8D56FC13B08, color, display_outside_vehicle, follow_player);

    for point in points {
        add_custom_route_point(*point);
    }
    set_custom_route_visible(true, radar_thickness, map_thickness);
}

pub fn clear_custom_route() {
    invoke!((), 0xE6DE0561D9232A64)
}

fn set_custom_route_visible(visible: bool, radar_thickness: u32, map_thickness: u32) {
    invoke!((), 0x900086F371220B6F, visible, radar_thickness, map_thickness)
}

fn add_custom_route_point(pos: Vector3<f32>) {
    invoke!((), 0xA905192A6781C41B, pos)
}

pub fn is_blip_route_found() -> bool {
    invoke!(bool, 0x869DAACBBE9FA006)
}

pub fn get_blip_route_length() -> u32 {
    invoke!(u32, 0xBBB45C3CF5C8AA85)
}

pub fn clear_player_waypoint() {
    invoke!((), 0xFF4FB7C8CDFA3DA7)
}

pub fn delete_waypoint() {
    invoke!((), 0xD8E694757BCEA8E9)
}

pub fn is_waypoint_active() -> bool {
    invoke!(bool, 0x1DD1F58F493F1DA5)
}

pub fn refresh_waypoint() {
    invoke!((), 0x81FA173F170560D1)
}

pub fn set_waypoint_locked(locked: bool) {
    invoke!((), 0x58FADDED207897DC, locked)
}

pub fn set_waypoint(pos: Vector2<f32>) {
    invoke!((), 0xFE43368D2AA4F2FC, pos)
}

pub fn get_ground_elevation(pos: Vector3<f32>, unknown: bool) -> Option<f32> {
    let mut elevation = 0.0;
    if invoke!(bool, 0xC906A7DAB05C8D2B, pos, &mut elevation, unknown) {
        Some(elevation)
    } else {
        None
    }
}

pub fn get_ground_elevation_and_normal(pos: Vector3<f32>) -> Option<(f32, Vector3<f32>)> {
    let mut elevation = 0.0;
    let mut normal = NativeVector3::zero();
    if invoke!(bool, 0x8BDC7BFC57A81E76, pos, &mut elevation, &mut normal) {
        Some((elevation, normal.into()))
    } else {
        None
    }
}

pub fn get_zone_name<'a>(pos: Vector3<f32>) -> &'a str {
    invoke!(&'a str, 0xCD90657D4C30E1CA, pos)
}