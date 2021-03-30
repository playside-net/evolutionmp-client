use std::cell::RefCell;
use std::collections::VecDeque;

use cgmath::{Vector2, Vector3};

use crate::{bind_fn_detour, bind_fn_detour_ip, class};
use crate::game::ped::Ped;
use crate::game::vehicle::Vehicle;
use crate::hash::Hash;
use crate::win::input::InputEvent;

class!(Event @EventVT {
    fn destructor() -> (),
    fn m_8() -> (),
    fn equals(other: *const Event) -> bool,
    fn get_id() -> u32,
    fn m_20() -> u32,
    fn m_28() -> u32,
    fn get_arguments(buffer: *mut *const (), len: usize) -> bool,
    fn m_38() -> bool,
    fn m_40(other: *const Event) -> bool;
});

impl Event {
    pub fn get_id(&self) -> u32 {
        (self.v_table.get_id)(self as _)
    }

    pub fn get_arguments(&self, buffer: *mut *const (), len: usize) -> bool {
        (self.v_table.get_arguments)(self as _, buffer, len)
    }
}

impl std::cmp::PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        (self.v_table.equals)(self as _, other as _)
    }
}

#[derive(Debug)]
pub enum ScriptEvent {
    ConsoleInput(String),
    UserInput(InputEvent)
}

pub struct EventPool {
    input: Vec<ScriptEvent>,
    pub(crate) output: VecDeque<ScriptEvent>
}

impl EventPool {
    pub fn new() -> EventPool {
        EventPool {
            input: Vec::new(),
            output: VecDeque::new()
        }
    }

    pub fn push_input(&mut self, event: ScriptEvent) {
        self.input.push(event)
    }

    pub fn push_output(&mut self, event: ScriptEvent) {
        self.output.push_back(event)
    }

    pub fn swap(&mut self) {
        self.input.clear();
        while let Some(event) = self.output.pop_front() {
            self.input.push(event)
        }
    }

    pub fn iterate<F>(&mut self, mut handler: F) where F: FnMut(&ScriptEvent) -> bool {
        self.input.retain(|i| !handler(i));
    }
}

bind_fn_detour!(CALL_EVENT, "81 BF ? ? 00 00 ? ? 00 00 75 ? 48 8B CF E8", -0x36, call_event, (&(), Option<&Event>) -> *mut ());

pub unsafe extern fn call_event(group: &(), event: Option<&Event>) -> *mut () {
    if let Some(event) = event {
        let mut arg_count = 0;
        let mut args = [std::ptr::null::<()>(); 48];
        for i in 0..48 {
            if event.get_arguments(args.as_mut_ptr(), i * std::mem::size_of::<*const ()>()) {
                arg_count = i;
                break;
            }
        }
        info!("Called event id {} ({} args: {:?})", event.get_id(), arg_count, &args[..arg_count]);
    }
    CALL_EVENT(group, event)
}

bind_fn_detour!(GET_EVENT_DATA, "48 85 C0 74 14 4C 8B 10", -28, get_event_data, (i32, i32, *mut i32, u32) -> bool);

pub unsafe extern fn get_event_data(group: i32, event: i32, args: *mut i32, arg_count: u32) -> bool {
    warn!("Getting event data for group {} id {} argc {}", group, event, arg_count);
    GET_EVENT_DATA(group, event, args, arg_count)
}

pub fn init() {
    info!("Initializing native events...");
    //lazy_static::initialize(&CALL_EVENT);
    //lazy_static::initialize(&GET_EVENT_DATA);
}
