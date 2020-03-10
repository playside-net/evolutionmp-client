use crate::pattern::{MemoryRegion, RageBox};
use crate::native::alloc::RageVec;
use crate::scripts::vehicle::ScriptVehicle;
use crate::hash::{Hash, Hashable};
use crate::win::thread::__readgsqword;
use crate::runtime::{Script, ScriptContainer, Runtime};
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::mem::MaybeUninit;

use crate::{bind_fn, bind_field_ip};
use std::ffi::CStr;
use std::alloc::Layout;
use std::ops::{Deref, DerefMut};

bind_field_ip!(THREAD_COLLECTION, "48 8B C8 EB 03 48 8B CB 48 8B 05", 11, RageVec<ManuallyDrop<Box<ScriptThread>>>);
bind_field_ip!(THREAD_ID, "89 15 ? ? ? ? 48 8B 0C D8", 2, u32);
bind_field_ip!(THREAD_COUNT, "FF 0D ? ? ? ? 48 8B F9", 2, u32);
bind_field_ip!(SCRIPT_MANAGER, "74 17 48 8B C8 E8 ? ? ? ? 48 8D 0D", 13, ScriptManager);

bind_fn!(SCRIPT_THREAD_INIT, "83 89 38 01 00 00 FF 83 A1 50 01 00 00 F0", 0, "thiscall", fn(*mut ScriptThread) -> ());
bind_fn!(SCRIPT_THREAD_KILL, "48 83 EC 20 48 83 B9 10 01 00 00 00 48 8B D9 74 14", -6, "thiscall", fn(*mut ScriptThread) -> ());
bind_fn!(SCRIPT_THREAD_TICK, "80 B9 46 01 00 00 00 8B FA 48 8B D9 74 05", -0xF, "thiscall", fn(*mut ScriptThread, u32) -> ThreadState);

pub(crate) fn pre_init() {
    lazy_static::initialize(&THREAD_COLLECTION);
    lazy_static::initialize(&THREAD_ID);
    lazy_static::initialize(&THREAD_COUNT);
    lazy_static::initialize(&SCRIPT_MANAGER);

    lazy_static::initialize(&SCRIPT_THREAD_INIT);
    lazy_static::initialize(&SCRIPT_THREAD_KILL);
    lazy_static::initialize(&SCRIPT_THREAD_TICK);
}

pub fn is_thread_pool_empty() -> bool {
    THREAD_COLLECTION.is_empty()
}

const ACTIVE_THREAD_TLS_OFFSET: usize = 0x830;

fn get_active_thread() -> *mut *mut Thread {
    unsafe {
        let module_tls = *(__readgsqword(88) as *mut *mut u8);
        module_tls.add(ACTIVE_THREAD_TLS_OFFSET).cast::<*mut Thread>()
    }
}

#[repr(C)]
pub struct ScriptManagerVTable {
    destructor: extern "C" fn(this: *mut ScriptManager),
    fn1:        extern "C" fn(this: *mut ScriptManager),
    fn2:        extern "C" fn(this: *mut ScriptManager),
    fn3:        extern "C" fn(this: *mut ScriptManager),
    fn4:        extern "C" fn(this: *mut ScriptManager),
    fn5:        extern "C" fn(this: *mut ScriptManager),
    fn6:        extern "C" fn(this: *mut ScriptManager),
    fn7:        extern "C" fn(this: *mut ScriptManager),
    fn8:        extern "C" fn(this: *mut ScriptManager),
    fn9:        extern "C" fn(this: *mut ScriptManager),
    attach:     extern "C" fn(this: *mut ScriptManager, thread: *mut ScriptThread),
}

#[repr(u32)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum ThreadState {
    Idle = 0,
    Running = 1,
    Killed = 2,
    Unknown3 = 3,
    Unknown4 = 4,
}

impl Default for ThreadState {
    fn default() -> Self {
        ThreadState::Idle
    }
}

#[repr(C)]
pub struct ScriptManager {
    v_table: ManuallyDrop<Box<ScriptManagerVTable>>
}

impl ScriptManager {
    pub fn attach(&mut self, script: &mut ScriptThread) {
        (self.v_table.attach)(self, script)
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ScriptThreadContext {
    id: u32,
    script_hash: Hash,
    state: ThreadState,
    ip: u32,
    frame_sp: u32,
    sp: u32,
    timer_a: u32,
    timer_b: u32,
    timer_c: u32,
    unk1: u32,
    unk2: u32,
    pad1: [u32; 13],
    set1: u32,
    pad2: [u32; 17]
}

#[repr(C)]
pub struct ThreadVTable {
    drop:   extern "C" fn(this: *mut ()),
    reset:  extern "C" fn(this: *mut (), id: u32, args: *const (), len: u32) -> ThreadState,
    run:    extern "C" fn(this: *mut (), ops: u32) -> ThreadState,
    tick:   extern "C" fn(this: *mut (), ops: u32) -> ThreadState,
    kill:   extern "C" fn(this: *mut ()),
    do_run: extern "C" fn(this: *mut ()),
    wut:    extern "C" fn(this: *mut ())
}

#[repr(C)]
pub struct Thread {
    v_table: ManuallyDrop<Box<ThreadVTable>>,
    context: ScriptThreadContext,
    stack: u64,
    pad1: u64,
    pad2: u64,
    sz_exit_message: *const c_char
}

#[repr(C)]
pub struct ScriptThread {
    parent: Thread,
    script_name: [u8; 64],
    script_handler: *const (),
    pad2: [u64; 5],
    flag1: u8,
    net_flag: u8,
    pad3: [u16; 11]
}

impl ScriptThread {
    pub fn new(name: &str, v_table: ThreadVTable) -> ScriptThread {
        assert!(name.len() < 64, "script name too long");
        let mut script_name = [0; 64];
        unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), script_name.as_mut_ptr(), name.len()) };
        ScriptThread {
            parent: Thread {
                v_table: ManuallyDrop::new(Box::new(v_table)),
                context: ScriptThreadContext {
                    script_hash: name.joaat(),
                    .. Default::default()
                },
                stack: 0,
                pad1: 0,
                pad2: 0,
                sz_exit_message: std::ptr::null()
            },
            script_name,
            script_handler: std::ptr::null(),
            pad2: [0; 5],
            flag1: 0,
            net_flag: 0,
            pad3: [0; 11]
        }
    }
}

#[repr(C)]
pub struct ScriptThreadRuntime {
    parent: ScriptThread,
    runtime: Runtime
}

macro_rules! vtable_fn {
    ($path:path) => {
        unsafe { std::mem::transmute($path as *const ()) }
    };
}

impl ScriptThreadRuntime {
    pub fn spawn(runtime: Runtime) {
        unsafe {
            let mut thread = ScriptThreadRuntime {
                parent: ScriptThread::new("runtime", ThreadVTable {
                    drop: vtable_fn!(Self::drop),
                    reset: vtable_fn!(Self::reset),
                    run: vtable_fn!(Self::run),
                    tick: vtable_fn!(Self::tick),
                    kill: vtable_fn!(Self::kill),
                    do_run: vtable_fn!(Self::do_run),
                    wut: vtable_fn!(Self::wut)
                }),
                runtime
            };
            let mut collection = THREAD_COLLECTION.as_mut();
            let mut slot = collection.iter()
                .enumerate()
                .find_map(|(i, t)| {
                    if t.context.id == 0 { Some(i) } else { None }
                })
                .unwrap_or(0);
            let thread_id = THREAD_ID.min(1);
            thread.reset(Hash(**THREAD_COUNT + 1), std::ptr::null(), 0);
            thread.context.id = thread_id;
            *THREAD_COUNT.as_mut() += 1;
            *THREAD_ID.as_mut() = thread_id + 1;
            collection[slot] = ManuallyDrop::new(std::mem::transmute(Box::new(thread)));
        }
    }

    unsafe extern "C" fn drop(&mut self) {
        std::ptr::drop_in_place(self as *mut _);
        std::alloc::dealloc(self as *mut Self as *mut _, Layout::new::<Self>())
    }

    extern "C" fn kill(&mut self) {
        SCRIPT_THREAD_KILL(&mut **self)
    }

    unsafe extern "C" fn run(&mut self, ops: u32) -> ThreadState {
        if self.script_handler.is_null() {
            SCRIPT_MANAGER.as_mut().attach(&mut **self);
            self.net_flag = 1;
        }
        let state = self.context.state;
        if state != ThreadState::Killed {
            let prev_thread = &mut **get_active_thread();
            *get_active_thread() = (self as *mut Self).cast();
            self.do_run();
            *get_active_thread() = prev_thread;
        }
        self.context.state
    }

    extern "C" fn reset(&mut self, hash: Hash, args: *const (), len: u32) -> ThreadState {
        self.context = ScriptThreadContext {
            state: ThreadState::Idle,
            script_hash: hash,
            unk1: std::u32::MAX,
            unk2: std::u32::MAX,
            set1: 1,
            .. Default::default()
        };
        SCRIPT_THREAD_INIT(&mut **self);
        self.sz_exit_message = b"Normal exit\0".as_ptr() as _;
        self.context.state
    }

    extern "C" fn tick(&mut self, ops: u32) -> ThreadState {
        SCRIPT_THREAD_TICK(&mut **self, ops)
    }

    extern "C" fn do_run(&mut self) {
        self.runtime.frame();
    }
}

impl Deref for ScriptThreadRuntime {
    type Target = ScriptThread;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl DerefMut for ScriptThreadRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parent
    }
}

impl Deref for ScriptThread {
    type Target = Thread;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl DerefMut for ScriptThread {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parent
    }
}