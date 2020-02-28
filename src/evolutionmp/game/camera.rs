use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;
use crate::native::{Addressable, NativeField};
use cgmath::{Vector3, Matrix3, Euler, Deg, Matrix4, SquareMatrix, Angle};

pub fn render_scripted(render: bool, transition_time: Option<u32>) {
    invoke!((), 0x07E5B515DB0636FC, render, transition_time.is_some(), transition_time.unwrap_or(0), true, false)
}

pub fn get_camera_type() -> u32 {
    invoke!(u32, 0x19CAFA3C87F7C2FF)
}

pub fn get_view_mode(camera_type: u32) -> u32 {
    invoke!(u32, 0xEE778F8C7E1142E2, camera_type)
}

pub fn get_gameplay_fov() -> f32 {
    invoke!(f32, 0x65019750A0324133)
}

pub fn get_gameplay_position() -> Vector3<f32> {
    invoke!(Vector3<f32>, 0x14D6F5678D8F1B37)
}

pub fn get_gameplay_rotation(order: u32) -> Vector3<f32> {
    invoke!(Vector3<f32>, 0x837765A25378F0BB, order)
}

pub fn rotation_to_direction(rot: Vector3<f32>) -> Vector3<f32> {
    let rot = Euler::new(Deg(rot.x), Deg(rot.y), Deg(rot.z));
    Vector3::new(
        -rot.z.sin() * rot.x.cos().abs(),
        rot.z.cos() * rot.x.cos().abs(),
        rot.x.sin()
    )
}

pub fn get_gameplay_direction() -> Vector3<f32> {
    rotation_to_direction(get_gameplay_rotation(2))
}

pub fn get_gameplay_relative_heading() -> f32 {
    invoke!(f32, 0x743607648ADD4587)
}

pub fn get_gameplay_relative_pitch() -> f32 {
    invoke!(f32, 0x3A6867B4845BEDA2)
}

pub struct Camera {
    handle: Handle
}

impl Camera {
    pub fn gameplay() -> Camera {
        Self::new_unset("DEFAULT_SCRIPTED_CAMERA").expect("gameplay camera missing")
    }

    pub fn new_unset<H>(name: H) -> Option<Camera> where H: Hashable {
        invoke!(Option<Camera>, 0x5E3CF89C6BCCA67D, name.joaat(), false)
    }

    pub fn new<H>(name: H, pos: Vector3<f32>, rotation: Vector3<f32>, fov: f32) -> Option<Camera> where H: Hashable {
        invoke!(Option<Camera>, 0x6ABFA3E16460F22D, name.joaat(), pos, rotation, fov, false, 2)
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
}

crate::impl_handle!(Camera);