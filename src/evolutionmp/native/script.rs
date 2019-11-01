use crate::invoke;
use crate::game::Handle;

pub unsafe fn terminate_all(name: &str) {
    invoke!((), 0x9DC711BC69C548DF, name)
}

pub unsafe fn get_active_thread() -> Handle {
    invoke!(Handle, 0xC30338E8088E2E21)
}

pub unsafe fn get_thread_name<'a>(thread: Handle) -> &'a str {
    invoke!(&str, 0x05A42BA9FC8DA96B, thread)
}

pub unsafe fn is_thread_active(thread: Handle) -> bool {
    invoke!(bool, 0x46E9AE36D8FA6417, thread)
}

pub unsafe fn thread_iterator_next() -> Handle {
    invoke!(Handle, 0x30B4FA1C82DD4B9F)
}

pub unsafe fn thread_iterator_reset() {
    invoke!((), 0xDADFADA5A20143A8)
}

pub unsafe fn terminate_active_thread() {
    invoke!((), 0x1090044AD1DA76FA)
}

pub unsafe fn terminate_thread(thread: Handle) {
    invoke!((), 0xC8B189ED9138BCD4, thread)
}