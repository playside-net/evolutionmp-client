use crate::invoke;

pub fn set_stunt_jumps_can_trigger(can_trigger: bool) {
    invoke!((), 0xD79185689F8FD5DF, can_trigger)
}

pub fn is_stunt_jump_in_progress() -> bool {
    invoke!(bool, 0x7A3F19700A4D0525)
}

pub fn cancel_stunt_jump() {
    invoke!((), 0xE6B7B0ACD4E4B75E)
}

pub fn get_mission_flag() -> bool {
    invoke!(bool, 0xA33CDCCDA663159E)
}

pub fn set_mission_flag(enabled: bool) {
    invoke!((), 0xC4301E5121A0ED73, enabled)
}

pub fn get_random_event_flag() -> bool {
    invoke!(bool, 0xD2D57F1D764117B1)
}

pub fn set_random_event_flag(enabled: bool) {
    invoke!((), 0x971927086CFD2158, enabled)
}

pub fn is_cutscene_active() -> bool {
    invoke!(bool, 0x991251AFC3981F84)
}

pub fn cancel_cutscene() {
    invoke!((), 0xD220BDD222AC4A1E)
}