use crate::pattern::MemoryRegion;
use crate::native::alloc::RageVec;
use crate::scripts::vehicle::ScriptVehicle;
use crate::hash::{Hash, Hashable};
use crate::win::thread::__readgsqword;
use crate::runtime::{Script, ScriptContainer, Runtime};
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use winapi::_core::mem::MaybeUninit;
use crate::get_game_state;
use crate::game::GameState;

static mut THREAD_COLLECTION: Option<ManuallyDrop<Box<RageVec<ManuallyDrop<Box<ScriptThread>>>>>> = None;
static mut THREAD_ID: *mut u32 = std::ptr::null_mut();
static mut THREAD_COUNT: *mut u32 = std::ptr::null_mut();
static mut SCRIPT_MANAGER: Option<ManuallyDrop<Box<ScriptManager>>> = None;
static mut SCRIPT_THREAD_INIT: Option<extern "C" fn(*mut ScriptThread)> = None;
static mut SCRIPT_THREAD_KILL: Option<extern "C" fn(*mut ScriptThread)> = None;
static mut SCRIPT_THREAD_TICK: Option<extern "C" fn(*mut ScriptThread, u32) -> ThreadState> = None;

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    THREAD_COLLECTION = Some(std::mem::transmute(
        mem.find("48 8B C8 EB 03 48 8B CB 48 8B 05")
            .next().expect("thread collection")
            .add(11).read_ptr(4).as_ptr()
    ));
    THREAD_ID = mem.find("89 15 ? ? ? ? 48 8B 0C D8")
        .next().expect("thread id")
        .add(2).read_ptr(4).get_mut();
    THREAD_COUNT = mem.find("FF 0D ? ? ? ? 48 8B F9")
        .next().expect("thread count")
        .add(2).read_ptr(4).get_mut();
    SCRIPT_MANAGER = Some(std::mem::transmute(
        mem.find("74 17 48 8B C8 E8 ? ? ? ? 48 8D 0D")
            .next().expect("script manager")
            .add(13).read_ptr(4).as_ptr()
    ));
    SCRIPT_THREAD_INIT = Some(std::mem::transmute(
        mem.find("83 89 38 01 00 00 FF 83 A1 50 01 00 00 F0")
            .next().expect("script_thread_init")
            .as_ptr()
    ));
    SCRIPT_THREAD_KILL = Some(std::mem::transmute(
        mem.find("48 83 EC 20 48 83 B9 10 01 00 00 00 48 8B D9 74 14")
            .next().expect("script_thread_kill")
            .offset(-6).as_ptr()
    ));
    SCRIPT_THREAD_TICK = Some(std::mem::transmute(
        mem.find("80 B9 46 01 00 00 00 8B FA 48 8B D9 74 05")
            .next().expect("script_thread_tick")
            .offset(-0xF).as_ptr()
    ));
}

pub fn is_thread_pool_empty() -> bool {
    unsafe { THREAD_COLLECTION.as_ref().unwrap().is_empty() }
}

fn get_active_thread() -> *mut *mut Thread {
    unsafe {
        let module_tls = *(__readgsqword(88) as *mut *mut u8);
        module_tls.add(0x830).cast::<*mut Thread>()
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
    pad1: [u8; 52],
    set1: u32,
    pad2: [u8; 68]
}

#[repr(C)]
pub struct ThreadVTable {
    do_run: extern "C" fn(this: *mut ()),
    reset:  extern "C" fn(this: *mut (), id: u32, args: *const (), len: u32) -> ThreadState,
    run:    extern "C" fn(this: *mut (), ops: u32) -> ThreadState,
    tick:   extern "C" fn(this: *mut (), ops: u32) -> ThreadState,
    kill:   extern "C" fn(this: *mut ())
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
    pad2: [u8; 40],
    flag1: u8,
    net_flag: u8,
    pad3: [u8; 22]
}

#[repr(C)]
pub struct ScriptThreadRuntime {
    parent: ScriptThread,
    runtime: Runtime
}

impl ScriptThreadRuntime {
    pub fn spawn(runtime: Runtime) {
        unsafe {
            let v_table = ManuallyDrop::new(Box::new(ThreadVTable {
                do_run: std::mem::transmute(Self::do_run as *const ()),
                reset: std::mem::transmute(Self::reset as *const ()),
                run: std::mem::transmute(Self::run as *const ()),
                tick: std::mem::transmute(Self::tick as *const ()),
                kill: std::mem::transmute(Self::kill as *const ())
            }));
            let mut script_name = [0; 64];
            std::ptr::copy_nonoverlapping(b"runtime".as_ptr(), script_name.as_mut_ptr(), 7);
            let mut thread = ScriptThreadRuntime {
                parent: ScriptThread {
                    parent: Thread {
                        v_table,
                        context: ScriptThreadContext {
                            id: 0,
                            script_hash: "runtime".joaat(),
                            state: ThreadState::Idle,
                            ip: 0,
                            frame_sp: 0,
                            sp: 0,
                            timer_a: 0,
                            timer_b: 0,
                            timer_c: 0,
                            unk1: 0,
                            unk2: 0,
                            pad1: [0; 52],
                            set1: 0,
                            pad2: [0; 68]
                        },
                        stack: 0,
                        pad1: 0,
                        pad2: 0,
                        sz_exit_message: std::ptr::null()
                    },
                    script_name,
                    script_handler: std::ptr::null(),
                    pad2: [0; 40],
                    flag1: 0,
                    net_flag: 0,
                    pad3: [0; 22]
                },
                runtime
            };
            let mut collection = THREAD_COLLECTION.as_mut().unwrap();
            let mut slot = 0;
            for thr in collection.iter() {
                let ctx = &thr.parent.context;
                if ctx.id == 0 {
                    break;
                }
                slot += 1;
            }
            thread.reset(Hash(*THREAD_COUNT + 1), std::ptr::null(), 0);
            if *THREAD_ID == 0 {
                *THREAD_ID += 1;
            }
            thread.parent.parent.context.id = *THREAD_ID;
            *THREAD_COUNT += 1;
            *THREAD_ID += 1;
            collection[slot] = ManuallyDrop::new(std::mem::transmute(Box::new(thread)));
        }
    }

    unsafe fn init(&mut self) {
        (SCRIPT_THREAD_INIT.unwrap())(&mut self.parent)
    }

    unsafe extern "C" fn kill(&mut self) {
        (SCRIPT_THREAD_KILL.unwrap())(&mut self.parent)
    }

    unsafe extern "C" fn run(&mut self, ops: u32) -> ThreadState {
        if self.parent.script_handler.is_null() {
            SCRIPT_MANAGER.as_mut().unwrap().attach(&mut self.parent);
            self.parent.net_flag = 1;
        }
        let state = self.parent.parent.context.state;
        if state != ThreadState::Killed {
            let prev_thread = &mut **get_active_thread();
            *get_active_thread() = &mut self.parent.parent;
            self.do_run();
            *get_active_thread() = prev_thread;
        }
        self.parent.parent.context.state
    }

    unsafe extern "C" fn reset(&mut self, hash: Hash, args: *const (), len: u32) -> ThreadState {
        self.parent.parent.context = std::mem::zeroed();
        {
            let mut context = &mut self.parent.parent.context;
            context.state = ThreadState::Idle;
            context.script_hash = hash;
            context.unk1 = std::u32::MAX;
            context.unk2 = std::u32::MAX;
            context.set1 = 1;
        }
        self.init();
        self.parent.parent.sz_exit_message = b"Normal exit\0".as_ptr() as _;
        self.parent.parent.context.state
    }

    unsafe extern "C" fn tick(&mut self, ops: u32) -> ThreadState {
        (SCRIPT_THREAD_TICK.unwrap())(&mut self.parent, ops)
    }

    unsafe extern "C" fn do_run(&mut self) {
        self.runtime.frame();
    }
}