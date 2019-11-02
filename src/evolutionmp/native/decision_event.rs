use crate::invoke;

pub unsafe fn suppress_shocking_events_next_frame() {
    invoke!((), 0x2F9A292AD0A3BD89)
}

pub unsafe fn suppress_agitation_events_next_frame() {
    invoke!((), 0x5F3B7749C112D552)
}