use crate::game::Handle;
use crate::invoke;
use std::time::Duration;

#[derive(Debug)]
pub struct ScriptThread {
    handle: Handle
}

impl ScriptThread {
    pub fn active() -> Option<ScriptThread> {
        invoke!(Option<ScriptThread>, 0xC30338E8088E2E21)
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

crate::impl_handle!(ScriptThread);

pub fn terminate_all(name: &str) {
    invoke!((), 0x9DC711BC69C548DF, name)
}

pub fn thread_iterator_next() -> Option<ScriptThread> {
    invoke!(Option<ScriptThread>, 0x30B4FA1C82DD4B9F)
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

pub fn get_all_threads() -> Vec<ScriptThread> {
    let mut threads = Vec::new();
    thread_iterator_reset();
    while let Some(thread) = thread_iterator_next() {
        threads.push(thread);
    }
    threads
}

pub fn wait(millis: u32) {
    //std::thread::sleep(Duration::from_millis(millis as u64));
    //invoke!((), 0x4EDE34FBADD967A6, millis);
}