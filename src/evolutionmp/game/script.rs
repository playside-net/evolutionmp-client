use crate::native::invoke;
use crate::game::Handle;
use crate::native::pool::Handleable;

pub struct ScriptThread {
    handle: Handle
}

impl ScriptThread {
    pub fn active() -> ScriptThread {
        invoke!(ScriptThread, 0xC30338E8088E2E21)
    }

    pub fn get_name(&self) -> &str {
        invoke!(&str, 0x05A42BA9FC8DA96B, self.handle)
    }

    pub fn is_active(&self) -> bool {
        invoke!(bool, 0x46E9AE36D8FA6417, self.handle)
    }

    pub fn terminate(&self) {
        invoke!((), 0xC8B189ED9138BCD4, self.handle)
    }
}

impl Handleable for ScriptThread {
    fn from_handle(handle: Handle) -> Option<Self> where Self: Sized {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }

    fn get_handle(&self) -> u32 {
        self.handle
    }
}

pub fn terminate_all(name: &str) {
    invoke!((), 0x9DC711BC69C548DF, name)
}

pub fn thread_iterator_next() -> Handle {
    invoke!(Handle, 0x30B4FA1C82DD4B9F)
}

pub fn thread_iterator_reset() {
    invoke!((), 0xDADFADA5A20143A8)
}

pub fn terminate_active_thread() {
    invoke!((), 0x1090044AD1DA76FA)
}

pub fn shutdown_loading_screen() {
    invoke!((), 0x078EBE9809CCD637)
}

pub fn mark_unused(script: &str) {
    invoke!((), 0xC90D2DCACD56184C, script)
}

pub fn force_cleanup(script: &str, flags: u32) {
    invoke!((), 0x4C68DDDDF0097317, script, flags)
}