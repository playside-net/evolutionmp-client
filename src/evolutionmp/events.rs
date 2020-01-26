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
use winapi::_core::mem::ManuallyDrop;

#[repr(C)]
struct EventVTable {
    destructor: extern "C" fn(this: *const Event),
    m_8: extern "C" fn(this: *const Event),
    equals: extern "C" fn(this: *const Event, other: *const Event) -> bool,
    get_id: extern "C" fn(this: *const Event) -> u32,
    m_20: extern "C" fn(this: *const Event) -> u32,
    m_28: extern "C" fn(this: *const Event) -> u32,
    get_arguments: extern "C" fn(this: *const Event, buffer: *mut *const (), len: usize) -> bool,
    m_38: extern "C" fn(this: *const Event) -> bool,
    m_40: extern "C" fn(this: *const Event, other: *const Event) -> bool
}

#[repr(C)]
pub struct Event {
    _vtable: ManuallyDrop<Box<EventVTable>>
}

impl Event {
    pub fn get_id(&self) -> u32 {
        (self._vtable.get_id)(self as _)
    }

    pub fn get_arguments(&self, buffer: *mut *const (), len: usize) -> bool {
        (self._vtable.get_arguments)(self as _, buffer, len)
    }
}

impl std::cmp::PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        (self._vtable.equals)(self as _, other as _)
    }
}

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

type CallEvent = extern "C" fn(*mut (), *mut Event) -> *mut ();
static mut CALL_EVENT: *const () = std::ptr::null();

pub unsafe extern "C" fn call_event(group: *mut (), event: *mut Event) -> *mut () {
    if !event.is_null() {
        let event = &*event;
        let mut arg_count = 0;
        let mut args = [std::ptr::null::<()>(); 48];
        for i in 0..48 {
            if event.get_arguments(args.as_mut_ptr(), i * std::mem::size_of::<*const ()>()) {
                arg_count = i;
                break;
            }
        }
        crate::info!("Called event id {} ({} args: {:?})", event.get_id(), arg_count, &args[..arg_count]);
    }
    let origin: CallEvent = std::mem::transmute(CALL_EVENT);
    origin(group, event)
}

pub unsafe fn init(mem: &MemoryRegion) {
    EVENTS.replace(Some(VecDeque::new()));

    let e =  mem.find("81 BF ? ? 00 00 ? ? 00 00 75 ? 48 8B CF E8")
        .next().expect("call_event")
        .offset(-0x36).get::<()>();

    let d = RawDetour::new(e, call_event as _).expect("detour creation failed");
    d.enable().expect("detour enabling failed");

    CALL_EVENT = d.trampoline() as _;
    std::mem::forget(d);

    native_event!(0xAF35D0D2583051B0, new_vehicle);
    native_event!(0xD49F9B0955C367DE, new_ped);
    native_event!(0xC20E50AA46D09CA8, task_enter_vehicle);
    native_event!(0xD3DBCE61A490BE02, task_leave_vehicle);
    native_event!(0xFE43368D2AA4F2FC, set_waypoint);
    native_event!(0x1D408577D440E81E, set_time_scale);
}
