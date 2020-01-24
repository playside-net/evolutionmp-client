use crate::pattern::MemoryRegion;
use crate::native::{NativeCallContext, ThreadSafe};
use crate::game::vehicle::Vehicle;
use crate::game::ped::Ped;
use crate::hash::{Hash, Hashable};
use crate::win::input::InputEvent;
use cgmath::{Vector3, Vector2};
use std::collections::VecDeque;
use winapi::_core::cell::RefCell;
use detour::RawDetour;
use std::ffi::CStr;

pub enum ScriptEvent {
    ConsoleInput(String),
    ConsoleOutput(String),
    NativeEvent(NativeEvent),
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

#[derive(Debug)]
pub enum NativeEvent {
    NewVehicle {
        model: Hash,
        pos: Vector3<f32>,
        heading: f32,
        is_network: bool,
        this_script_check: bool
    },
    NewPed {
        model: Hash,
        pos: Vector3<f32>,
        heading: f32,
        is_network: bool,
        this_script_check: bool
    },
    TaskEnterVehicle {
        ped: Ped,
        vehicle: Vehicle,
        timeout: u32,
        seat: i32,
        speed: f32,
        flag: i32,
        unknown: u32
    },
    TaskLeaveVehicle {
        ped: Ped,
        vehicle: Vehicle,
        flag: i32
    },
    SetPedWetness {
        ped: Ped,
        wetness: f32
    },
    SetWaypoint {
        pos: Vector2<f32>
    },
    SetTimeScale {
        scale: f32
    }
}

impl NativeEvent {
    pub fn new_vehicle(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::NewVehicle {
            model: args.read(),
            pos: args.read(),
            heading: args.read(),
            is_network: args.read(),
            this_script_check: args.read(),
        }
    }

    pub fn new_ped(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::NewPed {
            model: args.read(),
            pos: args.read(),
            heading: args.read(),
            is_network: args.read(),
            this_script_check: args.read(),
        }
    }

    pub fn task_enter_vehicle(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::TaskEnterVehicle {
            ped: args.read(),
            vehicle: args.read(),
            timeout: args.read(),
            seat: args.read(),
            speed: args.read(),
            flag: args.read(),
            unknown: args.read()
        }
    }

    pub fn task_leave_vehicle(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::TaskLeaveVehicle {
            ped: args.read(),
            vehicle: args.read(),
            flag: args.read()
        }
    }

    pub fn set_waypoint(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::SetWaypoint {
            pos: args.read()
        }
    }

    pub fn set_time_scale(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::SetTimeScale {
            scale: args.read()
        }
    }
}

pub(crate) static EVENTS: ThreadSafe<RefCell<Option<VecDeque<NativeEvent>>>> = ThreadSafe::new(RefCell::new(None));

pub fn push_native_event(event: NativeEvent) {
    if let Ok(mut events) = EVENTS.try_borrow_mut() {
        if let Some(events) = events.as_mut() {
            events.push_back(event);
        }
    }
}

macro_rules! native_event {
    ($hash:literal, $constructor:ident) => {
        crate::runtime::hook_native($hash, |context| {
            crate::events::push_native_event(NativeEvent::$constructor(context));
            crate::runtime::call_native_trampoline($hash, context);
        });
    };
}

pub unsafe fn init(mem: &MemoryRegion) {
    EVENTS.replace(Some(VecDeque::new()));

    native_event!(0xAF35D0D2583051B0, new_vehicle);
    native_event!(0xD49F9B0955C367DE, new_ped);
    native_event!(0xC20E50AA46D09CA8, task_enter_vehicle);
    native_event!(0xD3DBCE61A490BE02, task_leave_vehicle);
    native_event!(0xFE43368D2AA4F2FC, set_waypoint);
    native_event!(0x1D408577D440E81E, set_time_scale);
}
