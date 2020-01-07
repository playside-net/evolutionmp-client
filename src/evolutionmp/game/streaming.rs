use crate::native;
use crate::hash::{Hash, Hashable};
use crate::runtime::ScriptEnv;
use std::time::Duration;
use cgmath::Vector3;

pub fn stop_player_switch() {
    unsafe { native::streaming::stop_player_switch() }
}

pub fn load_scene(pos: Vector3<f32>) {
    unsafe { native::streaming::load_scene(pos) }
}

#[derive(Copy, Clone)]
pub struct Model {
    hash: Hash
}

impl Model {
    pub fn new<H>(hash: H) -> Model where H: Hashable {
        Model {
            hash: hash.joaat()
        }
    }

    pub fn is_loaded(&self) -> bool {
        unsafe { native::streaming::is_model_loaded(self.hash) }
    }

    pub fn is_collision_loaded(&self) -> bool {
        unsafe { native::streaming::is_model_collision_loaded(self.hash) }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { native::streaming::is_model_valid(self.hash) }
    }

    pub fn is_in_cd_image(&self) -> bool {
        unsafe { native::streaming::is_model_in_cd_image(self.hash) }
    }

    pub fn is_vehicle(&self) -> bool {
        unsafe { native::streaming::is_model_a_vehicle(self.hash) }
    }

    pub fn request(&self) {
        unsafe { native::streaming::request_model(self.hash) }
    }

    pub fn request_and_wait(&self, env: &mut ScriptEnv) {
        self.request();
        while !self.is_loaded() {
            env.wait(Duration::from_millis(0));
        }
    }

    pub fn request_collision(&self) {
        unsafe { native::streaming::request_model_collision(self.hash) }
    }

    pub fn mark_unused(&self) {
        unsafe { native::streaming::mark_model_unused(self.hash) }
    }
}

impl Hashable for Model {
    fn joaat(&self) -> Hash {
        self.hash
    }
}

pub struct AnimDict {
    name: String
}

impl AnimDict {
    pub fn new<N>(name: N) -> AnimDict where N: Into<String> {
        AnimDict {
            name: name.into()
        }
    }
}

impl AnimDict {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_valid(&self) -> bool {
        unsafe { native::streaming::is_anim_dict_valid(&self.name) }
    }

    pub fn is_loaded(&self) -> bool {
        unsafe { native::streaming::is_anim_dict_loaded(&self.name) }
    }

    pub fn request(&self) {
        unsafe { native::streaming::request_anim_dict(&self.name) }
    }

    pub fn mark_unused(&self) {
        unsafe { native::streaming::mark_anim_dict_unused(&self.name) }
    }
}