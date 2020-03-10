use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, PAGE_EXECUTE_READWRITE, IMAGE_OPTIONAL_HEADER64};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::minwindef::{DWORD, TRUE, HMODULE};
use std::any::Any;
use std::ptr::null_mut;
use std::time::{Duration, Instant};
use std::mem::ManuallyDrop;
use detour::RawDetour;
use std::ptr::replace;
use region::Protection;
use std::ops::{Deref, DerefMut};

pub const RET: u8 = 0xC3;
pub const NOP: u8 = 0x90;
pub const XOR_32_64: u8 = 0x31;

#[derive(Debug, Clone)]
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

    pub fn matches(&self, start: *mut u8) -> bool {
        for (i, n) in self.nibbles.iter().enumerate() {
            if let Some(n) = n {
                let o = unsafe { start.add(i).read() };
                if o != *n {
                    return false;
                }
            }
        }
        true
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for b in &self.nibbles {
            if let Some(b) = b {
                f.pad(&format!("{:016X} ", *b))?;
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

pub struct RegionIterator {
    pattern: Pattern,
    base: *mut u8,
    size: usize
}

impl RegionIterator {
    pub fn new<P>(pattern: P, region: &MemoryRegion) -> RegionIterator where P: Into<Pattern> {
        let pattern = pattern.into();
        RegionIterator {
            pattern,
            base: region.base,
            size: region.size
        }
    }
}

impl Iterator for RegionIterator {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let pattern_len = self.pattern.len();
        while self.size >= pattern_len {
            /*let readable = region::query_range(self.base, pattern_len)
                .unwrap().iter().all(|reg| reg.protection.contains(Protection::Read));*/
            if /*readable &&*/ self.pattern.matches(self.base) {
                let region = MemoryRegion {
                    base: self.base,
                    size: self.size
                };
                self.base = unsafe { self.base.add(pattern_len) };
                self.size -= pattern_len;
                return Some(region)
            } else {
                self.base = unsafe { self.base.add(1) };
            }
        }
        None
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
}

unsafe impl<T> Send for RageBox<T> {}
unsafe impl<T> Sync for RageBox<T> {}

#[derive(Clone)]
pub struct MemoryRegion {
    base: *mut u8,
    size: usize
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

    pub fn find<P>(&self, pattern: P) -> RegionIterator where P: Into<Pattern> {
        RegionIterator::new(pattern, &self)
    }

    pub fn find_str<S>(&self, str: S) -> RegionIterator where S: AsRef<str> {
        self.find(Pattern {
            nibbles: str.as_ref().as_bytes().iter().map(|b|Some(*b)).collect::<_>()
        })
    }

    pub fn find_await<P>(&self, pattern: P, sleep_ms: u64, timeout_ms: u64) -> Option<MemoryRegion> where P: Into<Pattern> + Copy {
        let start = Instant::now();
        loop {
            if (Instant::now() - start) >= Duration::from_millis(timeout_ms) {
                break None;
            }
            if let Some(region) = self.find(pattern).next() {
                break Some(region);
            }
            std::thread::sleep(Duration::from_millis(sleep_ms));
        }
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

    pub unsafe fn read_ptr(&self, offset: usize) -> MemoryRegion {
        self.add(offset).offset(*self.get::<i32>() as isize)
    }

    pub unsafe fn write_ptr(&self, ptr: *const ()) {
        let offset = (ptr as i64 - self.base as i64) as i32;
        *self.get_mut::<i32>() = offset;
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
                w.add(i).write(*b)
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