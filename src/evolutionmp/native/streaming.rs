use crate::invoke;
use crate::game::Handle;
use crate::hash::Hash;
use cgmath::Vector3;

pub fn set_vehicle_population_budget(budget: u32) {
    invoke!((), 0xCB9E1EB3BE2AF4E9, budget)
}

pub fn set_ped_population_budget(budget: u32) {
    invoke!((), 0x8C95333CFC3340F3, budget)
}

pub fn stop_player_switch() {
    invoke!((), 0x95C0A5BBDC189AA1)
}

pub fn load_scene(pos: Vector3<f32>) {
    invoke!((), 0x4448EB75B4904BDB, pos)
}

pub fn is_model_loaded(model: Hash) -> bool {
    invoke!(bool, 0x98A4EB5D89A0C952, model)
}

pub fn request_model(model: Hash) {
    invoke!((), 0x963D27A58DF860AC, model)
}

pub fn request_menu_ped_model(model: Hash) {
    invoke!((), 0xA0261AEF7ACFC51E, model)
}

pub fn request_models_in_room(interior: u32, room: &str) {
    invoke!((), 0x8A7A40100EDFEC58, interior, room)
}

pub fn mark_model_unused(model: Hash) {
    invoke!((), 0xE532F5D78798DAAB, model)
}

pub fn is_model_in_cd_image(model: Hash) -> bool {
    invoke!(bool, 0x35B9E0803292B641, model)
}

pub fn is_model_valid(model: Hash) -> bool {
    invoke!(bool, 0xC0296A2EDF545E92, model)
}

pub fn is_model_a_vehicle(model: Hash) -> bool {
    invoke!(bool, 0x19AAC8F07BFEC53E, model)
}

pub fn request_collision_at(pos: Vector3<f32>) {
    invoke!((), 0x07503F7948F491A7, pos)
}

pub fn request_model_collision(model: Hash) {
    invoke!((), 0x923CB32A3B874FCB, model)
}

pub fn is_model_collision_loaded(model: Hash) -> bool {
    invoke!(bool, 0x22CCA434E368F03A, model)
}

pub fn is_anim_dict_valid(dict: &str) -> bool {
    invoke!(bool, 0x2DA49C3B79856961, dict)
}

pub fn is_anim_dict_loaded(dict: &str) -> bool {
    invoke!(bool, 0xD031A9162D01088C, dict)
}

pub fn request_anim_dict(dict: &str) {
    invoke!((), 0xD3BD40951412FEF6, dict)
}

pub fn mark_anim_dict_unused(dict: &str) {
    invoke!((), 0xF66A602F829E2A06, dict)
}