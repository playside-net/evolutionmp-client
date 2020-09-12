use cgmath::{Angle, Deg, Euler, Vector3, Vector2, Zero, MetricSpace, Array};

use crate::game::Handle;
use crate::hash::{Hash, Hashable};
use crate::{invoke, invoke_option};
use crate::native::pool::Handleable;
use crate::client::native::pool::CCamera;

pub enum CameraShake {
    DeathFailInEffect,
    Drunk,
    Family5DrugTrip,
    Hand,
    Jolt,
    LargeExplosion,
    MediumExplosion,
    SmallExplosion,
    RoadVibration,
    SkyDiving,
    Vibrate,
}

impl CameraShake {
    pub fn get_name(&self) -> &'static str {
        match self {
            CameraShake::DeathFailInEffect => "DEATH_FAIL_IN_EFFECT_SHAKE",
            CameraShake::Drunk => "DRUNK_SHAKE",
            CameraShake::Family5DrugTrip => "FAMILY5_DRUG_TRIP_SHAKE",
            CameraShake::Hand => "HAND_SHAKE",
            CameraShake::Jolt => "JOLT_SHAKE",
            CameraShake::LargeExplosion => "LARGE_EXPLOSION_SHAKE",
            CameraShake::MediumExplosion => "MEDIUM_EXPLOSION_SHAKE",
            CameraShake::SmallExplosion => "SMALL_EXPLOSION_SHAKE",
            CameraShake::RoadVibration => "ROAD_VIBRATION_SHAKE",
            CameraShake::SkyDiving => "SKY_DIVING_SHAKE",
            CameraShake::Vibrate => "VIBRATE_SHAKE",
        }
    }
}

pub enum CameraType {
    DefaultScripted,
    DefaultAnimated,
    DefaultSpline,
    DefaultScriptedFly,
    TimedSpline,
}

impl CameraType {
    pub fn get_name(&self) -> &'static str {
        match self {
            CameraType::DefaultScripted => "DEFAULT_SCRIPTED_CAMERA",
            CameraType::DefaultAnimated => "DEFAULT_ANIMATED_CAMERA",
            CameraType::DefaultSpline => "DEFAULT_SPLINE_CAMERA",
            CameraType::DefaultScriptedFly => "DEFAULT_SCRIPTED_FLY_CAMERA",
            CameraType::TimedSpline => "TIMED_SPLINE_CAMERA",
        }
    }

    pub fn joaat(&self) -> Hash {
        self.get_name().joaat()
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum CameraViewMode {
    ThirdPersonClose,
    ThirdPersonMiddle,
    ThirdPersonFar,
    FirstPerson
}

impl Handleable for CameraViewMode {
    fn from_handle(handle: u32) -> Option<Self> where Self: Sized {
        if handle < 4 {
            unsafe { std::mem::transmute(handle) }
        } else {
            unreachable!("Invalid enum variant for CameraViewMode: {}", handle)
        }
    }

    fn get_handle(&self) -> u32 {
        *self as u32
    }
}

pub fn get_follow_ped_view_mode() -> CameraViewMode {
    invoke!(CameraViewMode, 0x8D4D46230B2C353A)
}

pub fn set_follow_ped_view_mode(mode: CameraViewMode) {
    invoke!((), 0x5A4F9EDF1673F704, mode as u32)
}

pub fn set_follow_ped(name: &str, ms: u32) {
    invoke!((), 0x44A113DD6FFC48D1, name, ms)
}

pub fn get_follow_ped_zoom() -> u32 {
    invoke!(u32, 0x33E6C8EFD0CD93E9)
}

pub fn is_follow_ped_active() -> bool {
    invoke!(bool, 0xC6D3D26810C8E0F9)
}

pub fn get_follow_vehicle_view_mode() -> CameraViewMode {
    invoke!(CameraViewMode, 0xA4FF579AC0E3AAAE)
}

pub fn set_follow_vehicle_view_mode(mode: CameraViewMode) {
    invoke!((), 0xAC253D7842768F48, mode as u32)
}

pub fn set_follow_vehicle(name: &str, ms: u32) {
    invoke!((), 0x44A113DD6FFC48D1, name, ms)
}

pub fn get_follow_vehicle_zoom() -> u32 {
    invoke!(u32, 0x33E6C8EFD0CD93E9)
}

pub fn set_follow_vehicle_zoom(zoom: u32) {
    invoke!((), 0x19464CB6E4078C8A, zoom)
}

pub fn is_follow_vehicle_active() -> bool {
    invoke!(bool, 0xCBBDE6D335D6D496)
}

pub fn render_first_person(render: bool, zoom: f32, mode: u32) {
    invoke!((), 0xC819F3CBB62BF692, render, zoom, mode)
}

pub fn fade_in(duration: u32) {
    invoke!((), 0xD4E8E24955024033, duration)
}

pub fn is_faded_in() -> bool {
    invoke!(bool, 0x5A859503B0C08678)
}

pub fn fade_out(duration: u32) {
    invoke!((), 0x891B5B39AC6302AF , duration)
}

pub fn is_faded_out() -> bool {
    invoke!(bool, 0xB16FCE9DDC7BA182)
}

pub fn render_scripted(render: bool, transition_time: Option<u32>) {
    invoke!((), 0x07E5B515DB0636FC, render, transition_time.is_some(), transition_time.unwrap_or(0), true, false)
}

pub fn get_camera_type() -> u32 {
    invoke!(u32, 0x19CAFA3C87F7C2FF)
}

pub fn get_view_mode(camera_type: u32) -> u32 {
    invoke!(u32, 0xEE778F8C7E1142E2, camera_type)
}

pub fn rotation_to_direction(rot: Vector3<f32>) -> Vector3<f32> {
    let rot = Euler::new(Deg(rot.x), Deg(rot.y), Deg(rot.z));
    Vector3::new(
        -rot.z.sin() * rot.x.cos().abs(),
        rot.z.cos() * rot.x.cos().abs(),
        rot.x.sin(),
    )
}

pub fn screen_from_world(world: Vector3<f32>) -> Option<Vector2<f32>> {
    let mut result = Vector2::zero();
    invoke_option!(result, 0x34E82F05DF2974F5, world, &mut result.x, &mut result.y)
}

pub fn screen_from_world_relative(world: Vector3<f32>) -> Option<Vector2<f32>> {
    screen_from_world(world).map(|pos| pos * 2.0 - Vector2::from_value(1.0))
}

pub fn world_from_screen(cam_pos: Vector3<f32>, cam_rot: Vector3<f32>, screen: Vector2<f32>) -> Vector3<f32> {
    let cam_forward = rotation_to_direction(cam_rot);
    let cam_right = rotation_to_direction(cam_rot - Vector3::unit_z() * 10.0);
    let cam_up = rotation_to_direction(cam_rot - Vector3::unit_x() * 10.0);
    let roll = cam_rot.y.to_radians();
    let cam_right_roll = cam_right * roll.cos() - cam_up * roll.sin();
    let cam_up_roll = cam_right * roll.sin() - cam_up * roll.cos();
    let end = cam_pos + cam_forward * 10.0;
    if let Some(target) = screen_from_world_relative(end + cam_right_roll + cam_up_roll) {
        if let Some(origin) = screen_from_world_relative(end) {
            let eps = 0.001;
            if target.distance2(origin) > eps * eps {
                let scale_x = (screen.x - origin.x) / (target.x - origin.x);
                let scale_y = (screen.y - origin.y) / (target.y - origin.y);
                return cam_pos + cam_forward * 10.0 + cam_right_roll * scale_x + cam_up_roll * scale_y;
            }
        }
    }
    end
}

pub struct GameplayCamera;

impl GameplayCamera {
    pub fn get_fov(&self) -> f32 {
        invoke!(f32, 0x65019750A0324133)
    }

    pub fn get_position(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x14D6F5678D8F1B37)
    }

    pub fn get_rotation(&self, order: u32) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x837765A25378F0BB, order)
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        rotation_to_direction(self.get_rotation(2))
    }

    pub fn get_relative_heading(&self) -> f32 {
        invoke!(f32, 0x743607648ADD4587)
    }

    pub fn get_relative_pitch(&self) -> f32 {
        invoke!(f32, 0x3A6867B4845BEDA2)
    }

    pub fn shake(&self, shake: CameraShake, amplitude: f32) {
        invoke!((), 0xFD55E49555E017CF, shake.get_name(), amplitude)
    }

    pub fn set_shake_amplitude(&self, amplitude: f32) {
        invoke!((), 0xA87E00932DB4D85D, amplitude)
    }

    pub fn is_shaking(&self) -> bool {
        invoke!(bool, 0x016C090630DF1F89)
    }

    pub fn stop_shaking(&self, instant: bool) {
        invoke!((), 0x0EF93E9F3D08C178, instant)
    }
}

#[derive(Debug)]
pub struct Camera {
    handle: Handle
}

impl Camera {
    pub fn new(ty: CameraType) -> Option<Camera> {
        invoke!(Option<Camera>, 0x5E3CF89C6BCCA67D, ty.joaat(), false)
    }

    pub fn new_parameterized(ty: CameraType, pos: Vector3<f32>, rotation: Vector3<f32>, fov: f32) -> Option<Camera> {
        invoke!(Option<Camera>, 0x6ABFA3E16460F22D, ty.joaat(), pos, rotation, fov, false, 2)
    }

    pub fn exists(&self) -> bool {
        invoke!(bool, 0xA7A932170592B50E, self.handle)
    }

    pub fn destroy(&self, check_this_script: bool) {
        invoke!((), 0x865908C81A2C22E9, self.handle, check_this_script)
    }

    pub fn get_position(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0xBAC038F7459AE5AE, self.handle)
    }

    pub fn set_position(&self, pos: Vector3<f32>) {
        invoke!((), 0x4D41783FB745E42E, self.handle, pos)
    }

    pub fn get_rotation(&self, order: u32) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x7D304C1C955E3E12, self.handle, order)
    }

    pub fn set_rotation(&self, rotation: Vector3<f32>, order: u32) {
        invoke!((), 0x85973643155D0B07, self.handle, rotation, order)
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        rotation_to_direction(self.get_rotation(2))
    }

    pub fn get_fov(&self) -> f32 {
        invoke!(f32, 0xC3330A45CCCDB26A, self.handle)
    }

    pub fn set_fov(&self, fov: f32) {
        invoke!((), 0xB13C14F66A00D047, self.handle, fov)
    }

    pub fn get_near_clip(&self) -> f32 {
        invoke!(f32, 0xC520A34DAFBF24B1, self.handle)
    }

    pub fn set_near_clip(&self, clip: f32) {
        invoke!((), 0xC7848EFCCC545182, self.handle, clip)
    }

    pub fn get_far_clip(&self) -> f32 {
        invoke!(f32, 0xB60A9CFEB21CA6AA, self.handle)
    }

    pub fn set_far_clip(&self, clip: f32) {
        invoke!((), 0xAE306F2A904BF86E, self.handle, clip)
    }

    pub fn is_active(&self) -> bool {
        invoke!(bool, 0xDFB2B516207D3534, self.handle)
    }

    pub fn set_active(&self, active: bool) {
        invoke!((), 0x026FB97D0A425F84, self.handle, active)
    }

    pub fn shake(&self, shake: CameraShake, amplitude: f32) {
        invoke!((), 0x6A25241C340D3822, self.handle, shake.get_name(), amplitude)
    }

    pub fn set_shake_amplitude(&self, amplitude: f32) {
        invoke!((), 0xD93DB43B82BC0D00, self.handle, amplitude)
    }

    pub fn is_shaking(&self) -> bool {
        invoke!(bool, 0x6B24BFE83A2BE47B, self.handle)
    }

    pub fn stop_shaking(&self, instant: bool) {
        invoke!((), 0xBDECF64367884AC3, self.handle, instant)
    }
}

crate::impl_native!(Camera, CCamera);