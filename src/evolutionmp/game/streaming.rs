use crate::native;
use crate::hash::{Hash, Hashable};
use crate::runtime::ScriptEnv;
use std::time::Duration;
use cgmath::Vector3;
pub use native::streaming::{
    stop_player_switch, load_scene
};

pub trait Resource {
    fn is_loaded(&self) -> bool;
    fn request(&self);
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

    pub fn is_collision_loaded(&self) -> bool {
        native::streaming::is_model_collision_loaded(self.hash)
    }

    pub fn is_valid(&self) -> bool {
        native::streaming::is_model_valid(self.hash)
    }

    pub fn is_in_cd_image(&self) -> bool {
        native::streaming::is_model_in_cd_image(self.hash)
    }

    pub fn is_vehicle(&self) -> bool {
        native::streaming::is_model_a_vehicle(self.hash)
    }

    pub fn request_collision(&self) {
        native::streaming::request_model_collision(self.hash)
    }

    pub fn mark_unused(&self) {
        native::streaming::mark_model_unused(self.hash)
    }
}

impl Resource for Model {
    fn is_loaded(&self) -> bool {
        native::streaming::is_model_loaded(self.hash)
    }

    fn request(&self) {
        native::streaming::request_model(self.hash)
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
        native::streaming::is_anim_dict_valid(&self.name)
    }

    pub fn mark_unused(&self) {
        native::streaming::mark_anim_dict_unused(&self.name)
    }
}

impl Resource for AnimDict {
    fn is_loaded(&self) -> bool {
        native::streaming::is_anim_dict_loaded(&self.name)
    }

    fn request(&self) {
        native::streaming::request_anim_dict(&self.name)
    }
}