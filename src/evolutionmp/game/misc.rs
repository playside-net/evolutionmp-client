use crate::invoke;

pub fn set_stunt_jumps_can_trigger(can_trigger: bool) {
    invoke!((), 0xD79185689F8FD5DF, can_trigger)
}