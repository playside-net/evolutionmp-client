use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::ops::{Add, Deref, DerefMut};
use std::os::raw::c_char;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

use crate::{bind_field, bind_field_ip, bind_fn, bind_fn_detour, bind_fn_detour_ip, class};
use crate::events::ScriptEvent;
use crate::hash::{Hash, Hashable};
use crate::native::alloc::RageVec;
use crate::native::ThreadSafe;
use crate::runtime::Script;
use crate::win::thread::__readgsqword;

lazy_static::lazy_static! {
    pub static ref LOADED_SCRIPTS: Mutex<Vec<ScriptThreadRuntime>> = Mutex::new(Vec::new());
    pub static ref EVENT_SENDERS: Mutex<Vec<Sender<ScriptEvent>>> = Mutex::new(Vec::new());
}

pub fn run<S>(name: &str, script: S) where S: Script + 'static {
    let mut loaded_scripts = LOADED_SCRIPTS.lock().unwrap();
    let mut event_senders = EVENT_SENDERS.lock().unwrap();
    let (sender, script) = ScriptThreadRuntime::new(name, Box::new(script));
    loaded_scripts.push(script);
    event_senders.push(sender);
}

bind_field!(SCRIPT_TLS_OFFSET, "48 8B 04 D0 4A 8B 14 00 48 8B 01 F3 44 0F 2C 42 20", -4, u32);

bind_field_ip!(THREAD_COLLECTION, "48 8B C8 EB 03 48 8B CB 48 8B 05", 11, RageVec<ManuallyDrop<Box<ScriptThread>>>);
bind_field_ip!(THREAD_COUNT, "FF 0D ? ? ? ? 48 8B F9", 2, u32);
bind_field_ip!(SCRIPT_MANAGER, "74 17 48 8B C8 E8 ? ? ? ? 48 8D 0D", 13, ScriptManager);

bind_fn!(SCRIPT_THREAD_INIT, "83 89 38 01 00 00 FF 83 A1 50 01 00 00 F0", 0, (&mut ScriptThread) -> ());
bind_fn!(SCRIPT_THREAD_KILL, "48 83 EC 20 48 83 B9 10 01 00 00 00 48 8B D9 74 14", -6, (&mut ScriptThread) -> ());
bind_fn!(SCRIPT_THREAD_TICK, "80 B9 46 01 00 00 00 8B FA 48 8B D9 74 05", -0xF, (&mut ScriptThread, u32) -> RageThreadState);

bind_fn_detour_ip!(SCRIPT_POST_INIT, "BA 2F 7B 2E 30 41 B8 0A", 11, script_post_init, (&(), Hash, u32) -> *mut u8);
bind_fn_detour!(SCRIPT_STARTUP, "83 FB FF 0F 84 D6 00 00 00", -0x37, script_startup, () -> ());
bind_fn_detour!(SCRIPT_RESET, "48 63 18 83 FB FF 0F 84 D6", -0x34, script_reset, () -> ());
bind_fn_detour!(SCRIPT_RUN, "48 83 EC 20 80 B9 46 01 00 00 00 8B FA", -0xB, script_run, (&'static mut ScriptThread, u32) -> RageThreadState);
bind_fn_detour!(SCRIPT_ACCESS, "74 3C 48 8B 01 FF 50 10 84 C0", -0x1A, script_access, (&'static mut ScriptThread, *mut ()) -> bool);


/**
    Probably something related to pool allocation/offsets ?
 */
unsafe extern fn script_post_init(arg: &(), ty: Hash, p3: u32) -> *mut u8 {
    let fn_name = crate::native::vtables::V_TABLES.get(&ty).map(|f| format!("{} ({})", f, ty)).unwrap_or_else(|| format!("({})", ty));
    let result = SCRIPT_POST_INIT(arg, ty, p3);
    if fn_name.contains("phMaterialMgr") {
        info!("called post_init on {:p}, {}, {} -> {:p}", arg, fn_name, p3, result);
    }

    /*let mut loaded_scripts = LOADED_SCRIPTS.lock().unwrap();

    for script in loaded_scripts.iter_mut() {
        if script.context.id == 0 {
            info!("Spawning own script {} on post_init", script.get_name().to_string_lossy());
            script.spawn();
        }
    }*/

    result
}

unsafe extern fn script_startup() {
    SCRIPT_STARTUP();
    let mut loaded_scripts = LOADED_SCRIPTS.lock().unwrap();

    for script in loaded_scripts.iter_mut() {
        if script.context.id == 0 {
            info!("Spawning own script {} on startup", script.get_name().to_string_lossy());
            script.spawn();
        }
    }
}

unsafe extern fn script_reset() {
    info!("Resetting GTA scripts");

    SCRIPT_RESET(); //Story mode only

    /*for thread in crate::game::script::get_all_threads() {
        warn!("Scr thread {}", thread.get_name());
    }*/

    info!("Now resetting owned scripts");

    let mut loaded_scripts = LOADED_SCRIPTS.lock().unwrap();

    for script in loaded_scripts.iter_mut() {
        script.reset(script.context.script_hash, std::ptr::null(), 0);
    }
}

unsafe extern fn script_run(script: &'static mut ScriptThread, ops: u32) -> RageThreadState {
    let mut loaded_scripts = LOADED_SCRIPTS.lock().unwrap();
    if let Some(s) = loaded_scripts.iter_mut().find(|s| s.context.id == script.context.id) {
        s.run(ops);
        return script.context.state;
    }
    RageThreadState::Killed
    //SCRIPT_RUN(script, ops)
}

unsafe extern fn script_access(script: &'static mut RageThread, unk: *mut ()) -> bool {
    info!("Script {} asked for access to {:p}", script.context.script_hash, unk);
    true
}

pub(crate) fn hook() {
    info!("Hooking scripts...");
    lazy_static::initialize(&SCRIPT_TLS_OFFSET);

    lazy_static::initialize(&THREAD_COLLECTION);
    lazy_static::initialize(&THREAD_COUNT);
    lazy_static::initialize(&SCRIPT_MANAGER);

    lazy_static::initialize(&SCRIPT_POST_INIT);
    lazy_static::initialize(&SCRIPT_STARTUP);
    lazy_static::initialize(&SCRIPT_RESET);
    lazy_static::initialize(&SCRIPT_RUN);
    //lazy_static::initialize(&SCRIPT_ACCESS);

    lazy_static::initialize(&SCRIPT_THREAD_INIT);
    lazy_static::initialize(&SCRIPT_THREAD_KILL);
    lazy_static::initialize(&SCRIPT_THREAD_TICK);
}

pub fn is_thread_pool_empty() -> bool {
    THREAD_COLLECTION.is_empty()
}

pub fn get_active_thread() -> *mut RageThread {
    unsafe {
        let module_tls = *(__readgsqword(88) as *mut *mut u8);
        *module_tls.add(**SCRIPT_TLS_OFFSET as usize).cast::<*mut RageThread>()
    }
}

pub fn set_active_thread(thread: *mut RageThread) {
    unsafe {
        let module_tls = *(__readgsqword(88) as *mut *mut u8);
        *module_tls.add(**SCRIPT_TLS_OFFSET as usize).cast::<*mut RageThread>() = thread;
    }
}

fn with_thread<A>(thread: &mut ScriptThreadRuntime, mut action: A) where A: FnMut(&mut ScriptThreadRuntime) {
    let old_thread = get_active_thread();
    set_active_thread((thread as *mut ScriptThreadRuntime).cast());
    action(thread);
    set_active_thread(old_thread);
}

#[repr(u32)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum RageThreadState {
    Idle,
    Running,
    Killed,
    Unknown3,
    Unknown4,
}

impl Default for RageThreadState {
    fn default() -> Self {
        RageThreadState::Idle
    }
}

class!(ScriptManager @ScriptManagerVT {
    fn destructor() -> (),
    fn fn1() -> (),
    fn fn2() -> (),
    fn fn3() -> (),
    fn fn4() -> (),
    fn fn5() -> (),
    fn fn6() -> (),
    fn fn7() -> (),
    fn fn8() -> (),
    fn fn9() -> (),
    fn attach(thread: *mut ScriptThread) -> ();
});

impl ScriptManager {
    pub fn attach(&mut self, script: &mut ScriptThread) {
        (self.v_table.attach)(self, script)
    }
}

#[repr(C)]
#[derive(Default)]
pub struct RageThreadContext {
    id: u32,
    pub(crate) script_hash: Hash,
    state: RageThreadState,
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
    pad2: [u32; 17],
}

class!(RageThread @RageThreadVTable {
    fn drop() -> (),
    fn reset(id: u32, args: *const (), len: u32) -> (),
    fn run(ops: u32) -> RageThreadState,
    fn tick(ops: u32) -> (),
    fn kill() -> (),
    fn frame() -> ();

    pub context: RageThreadContext,
    stack: u64,
    pad1: u64,
    pad2: u64,
    sz_exit_message: *const c_char
});

#[repr(C)]
pub struct ScriptThread {
    parent: RageThread,
    script_name: [u8; 64],
    script_handler: *const (),
    net_component: *const (),
    pad2: [u8; 24],
    net_id: u32,
    pad3: u32,
    flag1: bool,
    net_flag: bool,
    pad4: u16,
    pad5: [u8; 12],
    can_remove_blips_from_other_scripts: bool,
    pad6: [u8; 7],
}

impl ScriptThread {
    pub fn new(name: &str, v_table: RageThreadVTable) -> ScriptThread {
        assert!(name.len() < 64, "script name too long");
        let mut script_name = [0; 64];
        script_name[0..name.len()].copy_from_slice(name.as_bytes());
        ScriptThread {
            parent: RageThread {
                v_table: ManuallyDrop::new(Box::new(v_table)),
                context: RageThreadContext {
                    script_hash: name.joaat(),
                    ..Default::default()
                },
                stack: 0,
                pad1: 0,
                pad2: 0,
                sz_exit_message: std::ptr::null(),
            },
            script_name,
            script_handler: std::ptr::null(),
            net_component: std::ptr::null(),
            pad2: [0; 24],
            flag1: false,
            net_flag: false,
            pad4: 0,
            pad5: [0; 12],
            can_remove_blips_from_other_scripts: false,
            pad3: 0,
            net_id: 0,
            pad6: [0; 7],
        }
    }

    pub fn spawn(&mut self) {
        unsafe {
            let collection = THREAD_COLLECTION.as_mut();
            let slot = collection.iter()
                .position(|t| t.script_name == self.script_name)
                .or_else(|| collection.iter().position(|t| t.context.id == 0));

            if let Some(slot) = slot {
                let thread_id = THREAD_COUNT.max(1);
                self.context.id = thread_id;
                self.context.script_hash = Hash(THREAD_COUNT.add(1));
                *THREAD_COUNT.as_mut() += 1;
                collection[slot as usize] = ManuallyDrop::new(std::mem::transmute(self));
            }
        }
    }

    pub fn get_name<'a>(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(&self.script_name as *const u8 as _) }
    }
}

#[repr(C)]
pub struct ScriptThreadRuntime {
    parent: ThreadSafe<ScriptThread>,
    script: ThreadSafe<Box<dyn Script>>,
    receiver: Receiver<ScriptEvent>,
}

macro_rules! vtable_fn {
    ($path:path) => {
        unsafe { std::mem::transmute($path as *const ()) }
    };
}

impl ScriptThreadRuntime {
    pub fn new(name: &str, script: Box<dyn Script>) -> (Sender<ScriptEvent>, ScriptThreadRuntime) {
        assert_eq!(std::mem::size_of::<ScriptThread>(), 344, "script thread size is not 344 bytes");
        let (sender, receiver) = std::sync::mpsc::channel();
        (sender, ScriptThreadRuntime {
            parent: ThreadSafe::new(ScriptThread::new(&format!("emp:{}", name), RageThreadVTable {
                drop: vtable_fn!(Self::drop),
                reset: vtable_fn!(Self::reset),
                run: vtable_fn!(Self::run),
                tick: vtable_fn!(Self::tick),
                kill: vtable_fn!(Self::kill),
                frame: vtable_fn!(Self::frame),
            })),
            script: ThreadSafe::new(script),
            receiver,
        })
    }

    extern fn drop(self: Box<ScriptThreadRuntime>) {}

    extern fn kill(&mut self) {
        SCRIPT_THREAD_KILL(self)
    }

    extern fn run(&mut self, _ops: u32) -> RageThreadState {
        with_thread(self, move |script| {
            if script.context.state != RageThreadState::Killed {
                script.frame();
            }
        });
        self.context.state
    }

    extern fn reset(&mut self, hash: Hash, _args: *const (), _len: u32) -> RageThreadState {
        info!("Called reset on script {}", self.get_name().to_string_lossy());
        self.context = RageThreadContext {
            state: RageThreadState::Idle,
            script_hash: hash,
            unk1: u32::MAX,
            unk2: u32::MAX,
            set1: 1,
            ..Default::default()
        };
        SCRIPT_THREAD_INIT(self);
        self.net_flag = true;
        self.can_remove_blips_from_other_scripts = true;
        self.sz_exit_message = c_str!("Normal exit").as_ptr();
        if self.context.id == 0 {
            self.context.id = **THREAD_COUNT;
            unsafe { *THREAD_COUNT.as_mut() += 1; }
        }
        unsafe { SCRIPT_MANAGER.as_mut().attach(self); }
        self.context.state
    }

    extern fn tick(&mut self, ops: u32) -> RageThreadState {
        SCRIPT_THREAD_TICK(self, ops)
    }

    extern fn frame(&mut self) {
        self.script.frame();
        while let Ok(event) = self.receiver.try_recv() {
            self.script.event(event);
        }
    }
}

extern {
    #[link_name = "llvm.returnaddress"]
    fn return_address(param: i32) -> *const u8;
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
    type Target = RageThread;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl DerefMut for ScriptThread {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parent
    }
}