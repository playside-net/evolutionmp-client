use std::any::Any;
use std::ptr::null_mut;
use std::time::{Duration, Instant};
use std::mem::ManuallyDrop;
use std::ptr::replace;
use std::ops::{Deref, DerefMut};
use std::collections::HashMap;
use std::fs::File;
use std::sync::Mutex;
use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, PAGE_EXECUTE_READWRITE, IMAGE_OPTIONAL_HEADER64, PAGE_READONLY, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_NOACCESS, MEM_IMAGE};
use winapi::um::memoryapi::{VirtualProtect, VirtualQueryEx, VirtualProtectEx, ReadProcessMemory, VirtualQuery};
use winapi::um::libloaderapi::{GetModuleHandleA, GetModuleFileNameW};
use winapi::shared::minwindef::{DWORD, TRUE, HMODULE, MAX_PATH};
use detour::RawDetour;
use serde_derive::{Serialize, Deserialize};
use crate::launcher_dir;
use std::path::{Path, PathBuf};
use winapi::um::sysinfoapi::{SYSTEM_INFO, GetSystemInfo};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::errhandlingapi::GetLastError;

pub const RET: u8 = 0xC3;
pub const NOP: u8 = 0x90;
pub const XOR_32_64: u8 = 0x31;

lazy_static::lazy_static! {
    pub static ref CACHE: PatternCache = PatternCache::new();
}

#[derive(Serialize, Deserialize)]
pub struct PatternCache {
    inner: Mutex<HashMap<Pattern, Vec<u64>>>
}

impl PatternCache {
    fn new() -> PatternCache {
        PatternCache {
            inner: Mutex::new(HashMap::new())
        }
    }

    fn get_file(&self) -> PathBuf {
        launcher_dir().join("hints.dat")
    }

    fn get(&self, pattern: &Pattern, occurrence: usize) -> Option<MemoryRegion> {
        let mut cache = self.inner.lock().unwrap();
        if let Some(entry) = cache.get(pattern) {
            if let Some(offset) = entry.get(occurrence).cloned() {
                let mem = &crate::native::MEM;
                let base = mem.base as u64 + offset;
                let size = mem.size as u64 - offset;
                return Some(MemoryRegion {
                    base: base as _,
                    size: size as _
                });
            }
        }
        None
    }

    fn set(&self, pattern: &Pattern, region: &MemoryRegion) {
        let mut cache = self.inner.lock().unwrap();
        let mem = &crate::native::MEM;
        let offset = region.base as u64 - mem.base as u64;
        if let Some(mut entry) = cache.get_mut(pattern) {
            entry.push(offset);
        } else {
            cache.insert(pattern.clone(), vec![offset]);
        }
    }

    pub(crate) fn load(&self) {
        let mut file = self.get_file();
        if file.exists() {
            crate::info!("Loading hints...");
            let mut file = File::open(&file)
                .expect("error opening mem hints");
            let hints = bincode::deserialize_from::<_, HashMap<Pattern, Vec<u64>>>(&mut file)
                .expect("error reading mem hints");
            let mut cache = self.inner.lock().unwrap();
            *cache = hints;
        }
    }

    pub(crate) fn save(&self) {
        let mut file = self.get_file();
        if !file.exists() {
            crate::info!("Saving hints...");
            let mut file = File::create(&file)
                .expect("error creating mem hints");
            let mut cache = self.inner.lock().unwrap();
            bincode::serialize_into(&mut file, &mut *cache)
                .expect("error writing mem hints");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Pattern {
    nibbles: Vec<Option<u8>>
}

impl Pattern {
    pub fn compile(pattern: &str) -> Pattern {
        let mut nibbles = Vec::new();
        for b in pattern.split(" ") {
            if b == "?" || b == "??" {
                nibbles.push(None);
            } else {
                let b = u8::from_str_radix(b, 16).expect(&format!("Invalid pattern symbol: {}", b));
                nibbles.push(Some(b))
            }
        }
        Pattern { nibbles }
    }

    pub fn len(&self) -> usize {
        self.nibbles.len()
    }

    pub fn scan(&self, buf: &[u8]) -> Option<usize> {
        let pattern_len = self.nibbles.len();
        for i in 0..(buf.len() - pattern_len) {
            let mut found = true;
            for j in 0..pattern_len {
                if let Some(m) = self.nibbles[j] {
                    if m != buf[i + j] {
                        found = false;
                        break;
                    }
                }
            }
            if found {
                return Some(i);
            }
        }
        None
    }

    pub unsafe fn find(&self, region: &MemoryRegion) -> Option<MemoryRegion> {
        crate::info!("Searching for pattern {}", self);
        let mut sys_info = SYSTEM_INFO::default();
        GetSystemInfo(&mut sys_info);
        let end = sys_info.lpMaximumApplicationAddress;
        let mut current_chunk = std::ptr::null_mut();
        let mut bytes_read = 0;

        while current_chunk < end {
            let mut mbi = MEMORY_BASIC_INFORMATION::default();
            let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();

            let process = GetCurrentProcess();
            let hr = GetLastError();
            if VirtualQuery(current_chunk, &mut mbi, mbi_size) == 0 {
                return None;
            }

            if mbi.State == MEM_COMMIT && mbi.Protect != PAGE_NOACCESS && mbi.Type == MEM_IMAGE {
                let mut name = [0; MAX_PATH];
                let len = GetModuleFileNameW(mbi.AllocationBase.cast(), name.as_mut_ptr(), MAX_PATH as u32);
                if len != 0 {
                    let name = widestring::WideCStr::from_ptr_with_nul(name.as_ptr(), len as usize).to_string_lossy();
                    crate::info!("Reading chunk of size {} and type MEM_IMAGE in module {}", mbi.RegionSize, name);
                } else {
                    crate::info!("Reading chunk of size {} and type MEM_IMAGE", mbi.RegionSize);
                }
                let mut buffer = Vec::with_capacity(mbi.RegionSize);
                buffer.extend(std::iter::repeat(0u8).take(mbi.RegionSize));
                let mut old_protect = 0;
                if VirtualProtect(mbi.BaseAddress, mbi.RegionSize, PAGE_EXECUTE_READWRITE, &mut old_protect) == TRUE {
                    ReadProcessMemory(process, mbi.BaseAddress, buffer.as_mut_ptr().cast(), mbi.RegionSize, &mut bytes_read);
                    VirtualProtect(mbi.BaseAddress, mbi.RegionSize, old_protect, &mut old_protect);
                    if let Some(index) = self.scan(&buffer[0..bytes_read]) {
                        let base = current_chunk.add(index).cast();
                        let offset = base as u64 - mbi.AllocationBase as u64;
                        crate::info!("Found pattern {} at address {:p} offset {:X}", self, base, offset);
                        return Some(MemoryRegion {
                            base,
                            size: bytes_read - index
                        });
                    }
                }
            }
            current_chunk = current_chunk.add(mbi.RegionSize);
        }

        None
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for b in &self.nibbles {
            if let Some(b) = b {
                f.pad(&format!("{:02X} ", *b))?;
            } else {
                f.pad("? ")?;
            }
        }
        Ok(())
    }
}

impl<S> From<S> for Pattern where S: AsRef<str> {
    fn from(s: S) -> Self {
        Pattern::compile(s.as_ref())
    }
}

#[repr(transparent)]
pub struct RageBox<T> {
    ptr: *mut T
}

impl<T> Deref for RageBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> AsRef<T> for RageBox<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> RageBox<T> {
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub unsafe fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.ptr }
    }

    pub fn cloned(&self) -> RageBox<T> {
        RageBox {
            ptr: self.ptr
        }
    }
}

unsafe impl<T> Send for RageBox<T> {}
unsafe impl<T> Sync for RageBox<T> {}

#[derive(Clone)]
pub struct MemoryRegion {
    pub base: *mut u8,
    pub size: usize
}

impl MemoryRegion {
    pub fn with_size<S>(size: S) -> MemoryRegion where S: FnOnce(IMAGE_OPTIONAL_HEADER64) -> u32 {
        unsafe {
            let handle = GetModuleHandleA(null_mut());
            let lfa = handle.offset(std::mem::transmute::<HMODULE, *mut IMAGE_DOS_HEADER>(handle).read().e_lfanew as isize);
            let size = size(std::mem::transmute::<*mut (), *mut IMAGE_NT_HEADERS64>(lfa as *mut ()).read().OptionalHeader);
            MemoryRegion {
                base: handle as *mut _,
                size: size as usize
            }
        }
    }

    #[inline]
    pub fn image() -> MemoryRegion {
        Self::with_size(|header| header.SizeOfImage)
    }

    #[inline]
    pub fn code() -> MemoryRegion {
        Self::with_size(|header| header.SizeOfCode)
    }

    pub fn find<P>(&self, pattern: P) -> Option<MemoryRegion> where P: Into<Pattern> {
        unsafe { pattern.into().find(self) }
    }

    pub fn find_str<S>(&self, str: S) -> Option<MemoryRegion> where S: AsRef<str> {
        self.find(Pattern {
            nibbles: str.as_ref().as_bytes().iter().map(|b|Some(*b)).collect::<_>()
        })
    }

    pub fn contains(&self, address: *mut u8) -> bool {
        (self.base as usize) > (address as usize) && (address as usize) < (self.base as usize + self.size)
    }

    pub unsafe fn add(&self, offset: usize) -> MemoryRegion {
        MemoryRegion {
            base: self.base.add(offset),
            size: self.size - offset
        }
    }

    pub unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(self.base as _, self.size)
    }

    pub unsafe fn read_ptr(&self, offset: usize) -> MemoryRegion {
        self.add(offset).offset(*self.get::<i32>() as isize)
    }

    pub unsafe fn write_ptr(&self, ptr: *const ()) {
        let offset = (ptr as i64 - self.base as i64) as i32;
        *self.get_mut::<i32>() = offset;
    }

    pub unsafe fn offset_to(&self, target: *mut ()) -> MemoryRegion {
        let offset = target as isize - self.base as isize;
        self.offset(offset)
    }

    pub unsafe fn offset(&self, offset: isize) -> MemoryRegion {
        MemoryRegion {
            base: self.base.offset(offset),
            size: (self.size as isize - offset) as usize
        }
    }

    pub unsafe fn translate(mut self, from: MemoryRegion, to: MemoryRegion) -> MemoryRegion {
        self.base = to.offset(self.base as isize - from.base as isize).base;
        self
    }

    pub unsafe fn write_bytes(&self, bytes: &[u8]) -> bool {
        self.write(bytes.len(), |w| {
            for (i, b) in bytes.iter().enumerate() {
                w.add(i).write_unaligned(*b)
            }
        })
    }

    pub unsafe fn write<F>(&self, size: usize, writer: F) -> bool where F: Fn(*mut u8) {
        let mut old_mode: DWORD = 0;
        if self.protect(size, PAGE_EXECUTE_READWRITE, &mut old_mode) {
            writer(self.base);
            self.protect(size, old_mode, &mut 0);
            true
        } else {
            false
        }
    }

    pub unsafe fn nop(&self, size: usize) -> bool {
        self.write(size, |m| m.write_bytes(NOP, size))
    }

    pub unsafe fn replace<P>(&self, pattern: P) where P: Into<Pattern> {
        for (i, b) in pattern.into().nibbles.iter().map(|n|n.unwrap()).enumerate() {
            self.base.add(i).write(b)
        }
    }

    pub unsafe fn protect(&self, size: usize, mode: DWORD, old_mode: &mut DWORD) -> bool {
        VirtualProtect(self.base as *mut _, size, mode, old_mode) == TRUE
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.base
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.base
    }

    pub fn get_mut<T>(&self) -> *mut T {
        self.base.cast()
    }

    pub fn get<T>(&self) -> *const T {
        self.base.cast()
    }

    pub unsafe fn get_box<T>(&self) -> RageBox<T> {
        RageBox {
            ptr: self.base.cast()
        }
    }

    pub unsafe fn get_call(&self) -> *const () {
        self.add(1).read_ptr(4).get()
    }

    pub unsafe fn detour(&self, replacement: *const ()) -> *const () {
        let old = self.as_ptr() as *const ();
        let detour = RawDetour::new(old, replacement).expect("detour creation failed");
        detour.enable().expect("detour enabling failed");
        let old = detour.trampoline() as *const _;
        std::mem::forget(detour);
        old
    }

    pub unsafe fn detour_ip(&self, replacement: *const ()) -> *const () {
        let old = self.get_call();
        let detour = RawDetour::new(old, replacement).expect("detour creation failed");
        detour.enable().expect("detour enabling failed");
        let old = detour.trampoline() as *const _;
        std::mem::forget(detour);
        old
    }

    pub unsafe fn jump(&self, replacement: *const ()) -> *const () {
        let old = self.get_call();
        *self.get_mut::<u8>() = 0xE9;
        self.add(1).write_ptr(replacement);
        old
    }
}

unsafe impl Sync for MemoryRegion {}
unsafe impl Send for MemoryRegion {}

impl std::fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..self.size {
            f.pad(&format!("{:016X} ", unsafe { self.base.add(i).read() }))?;
        }
        Ok(())
    }
}