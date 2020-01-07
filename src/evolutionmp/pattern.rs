use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, PAGE_EXECUTE_READWRITE};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::minwindef::{DWORD, TRUE, HMODULE};
use std::any::Any;
use std::ptr::null_mut;
use std::time::{Duration, Instant};

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
    current: *mut u8,
    len: usize
}

impl RegionIterator {
    pub fn new<P>(pattern: P, region: &MemoryRegion) -> RegionIterator where P: Into<Pattern> {
        let pattern = pattern.into();
        RegionIterator {
            pattern,
            current: region.base,
            len: region.size
        }
    }
}

impl Iterator for RegionIterator {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let pattern_len = self.pattern.len();
        while self.len >= pattern_len {
            if self.pattern.matches(self.current) {
                let region = MemoryRegion {
                    base: self.current,
                    size: self.len //FIXME pattern_len
                };
                self.current = unsafe { self.current.add(pattern_len) };
                self.len -= pattern_len;
                return Some(region)
            } else {
                self.current = unsafe { self.current.add(1) };
                self.len -= 1;
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct MemoryRegion {
    base: *mut u8,
    size: usize
}

impl MemoryRegion {
    pub fn image() -> MemoryRegion {
        unsafe {
            let handle = GetModuleHandleA(null_mut());
            let lfa = handle.offset(std::mem::transmute::<HMODULE, *mut IMAGE_DOS_HEADER>(handle).read().e_lfanew as isize);
            let size = std::mem::transmute::<*mut (), *mut IMAGE_NT_HEADERS64>(lfa as *mut ()).read().OptionalHeader.SizeOfImage;
            MemoryRegion {
                base: handle as *mut _,
                size: size as usize
            }
        }
    }

    pub fn code() -> MemoryRegion {
        unsafe {
            let handle = GetModuleHandleA(null_mut());
            let lfa = handle.offset(std::mem::transmute::<HMODULE, *mut IMAGE_DOS_HEADER>(handle).read().e_lfanew as isize);
            let size = std::mem::transmute::<*mut (), *mut IMAGE_NT_HEADERS64>(lfa as *mut ()).read().OptionalHeader.SizeOfCode;
            MemoryRegion {
                base: handle as *mut _,
                size: size as usize
            }
        }
    }

    pub fn find<P>(&self, pattern: P) -> RegionIterator where P: Into<Pattern> {
        RegionIterator::new(pattern, &self)
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

    pub unsafe fn nop(&self, size: usize) -> bool {
        let mut old_mode: DWORD = 0;
        if self.protect(size, PAGE_EXECUTE_READWRITE, &mut old_mode) {
            self.base.write_bytes(0x90, size);
            self.protect(size, old_mode, &mut 0);
            true
        } else {
            false
        }
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

    pub unsafe fn get_mut<T>(&self) -> *mut T {
        self.base.cast()
    }

    pub unsafe fn get_box<T>(&self) -> Box<T> {
        Box::from_raw(self.base.cast())
    }

    pub unsafe fn get<T>(&self) -> *const T {
        self.base.cast()
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