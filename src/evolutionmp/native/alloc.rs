use crate::win::thread::__readgsqword;
use crate::pattern::MemoryRegion;
use crate::native::ThreadSafe;
use std::mem::ManuallyDrop;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use winapi::shared::minwindef::DWORD;
use winapi::um::winnt::HANDLE;
use winapi::um::processthreadsapi::GetThreadId;

static GLOBAL: ThreadSafe<Cell<*mut SysAlloc>> = ThreadSafe::new(Cell::new(std::ptr::null_mut()));
static TLS_OFFSET: ThreadSafe<Cell<u32>> = ThreadSafe::new(Cell::new(0));

unsafe fn get_global_alloc_ptr() -> *mut *mut SysAlloc {
    (*(__readgsqword(88) as *const usize) + TLS_OFFSET.get() as usize) as *mut *mut SysAlloc
}

pub unsafe fn reassign() {
    assert!(!GLOBAL.get().is_null(), "global alloc is null");
    *get_global_alloc_ptr() = GLOBAL.get();
    *get_global_alloc_ptr().offset(-1) = GLOBAL.get();
}

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    crate::info!("tls_offset");
    TLS_OFFSET.replace(*mem.find("B9 ? ? ? ? 48 8B 0C 01 45 33 C9 49 8B D2 48")
        .next().expect("alloc tls offset")
        .add(1).get());
}

#[repr(C)]
pub struct SysAllocVTable {
    destructor:             extern "C" fn(this: *mut SysAlloc),
    set_quit_on_fail:       extern "C" fn(this: *mut SysAlloc, quit: bool),
    get_unk:                extern "C" fn(this: *mut SysAlloc),
    allocate:               extern "C" fn(this: *mut SysAlloc, size: usize, align: usize, sub_alloc: u32) -> *mut (),
    try_allocate:           extern "C" fn(this: *mut SysAlloc, size: usize, align: usize, sub_alloc: u32) -> *mut (),
    free:                   extern "C" fn(this: *mut SysAlloc, ptr: *mut ()),
    try_free:               extern "C" fn(this: *mut SysAlloc, ptr: *mut ()),
    resize:                 extern "C" fn(this: *mut SysAlloc, ptr: *mut (), size: usize),
    get_allocator:          extern "C" fn(this: *mut SysAlloc, alloc: u32) -> *const SysAlloc,
    get_allocator_mut:      extern "C" fn(this: *mut SysAlloc, alloc: u32) -> *mut SysAlloc,
    get_ptr_owner:          extern "C" fn(this: *mut SysAlloc, ptr: *mut ()) -> *mut SysAlloc,
    get_size:               extern "C" fn(this: *mut SysAlloc, ptr: *mut ()) -> usize

}

#[repr(C)]
pub struct SysAlloc {
    v_table: ManuallyDrop<Box<SysAllocVTable>>
}

impl SysAlloc {
    pub unsafe fn global() -> ManuallyDrop<Box<SysAlloc>> {
        let alloc = *get_global_alloc_ptr();
        if alloc.is_null() {
            reassign();
        }
        ManuallyDrop::new(std::mem::transmute(alloc))
    }

    pub unsafe fn allocate<T>(&mut self, size: usize, offset: usize, sub_alloc: u32) -> *mut T where T: Sized {
        (self.v_table.allocate)(self, size * std::mem::size_of::<T>(), offset, sub_alloc) as *mut T
    }

    pub unsafe fn free<T>(&mut self, ptr: *mut T) {
        (self.v_table.free)(self, ptr as _)
    }
}

#[repr(C)]
pub struct RageVec<T> {
    ptr: *mut T,
    len: u16,
    capacity: u16
}

impl<T> RageVec<T> {
    pub fn new() -> RageVec<T> {
        RageVec {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0
        }
    }

    pub fn with_capacity(capacity: u16) -> RageVec<T> {
        unsafe {
            RageVec {
                ptr: SysAlloc::global().allocate(capacity as usize, 16, 0),
                len: 0,
                capacity
            }
        }
    }

    #[inline]
    pub fn len(&self) -> u16 {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> u16 {
        self.capacity
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn truncate(&mut self, len: u16) {
        unsafe {
            if len > self.len {
                return;
            }
            let s = self.get_unchecked_mut(len as usize ..) as *mut _;
            self.len = len;
            std::ptr::drop_in_place(s);
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn reserve(&mut self, count: u16) {
        let requested_capacity = self.len + count;
        if requested_capacity >= self.capacity {
            let new_capacity = std::cmp::max(self.capacity * 2, requested_capacity);
            unsafe {
                let ptr = SysAlloc::global().allocate(new_capacity as usize, 16, 0);
                if !self.ptr.is_null() {
                    std::ptr::copy_nonoverlapping(self.ptr, ptr, self.len as usize);
                }
            }
            self.capacity = new_capacity;
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.reserve(1);
        }
        unsafe {
            let end = self.as_mut_ptr().add(self.len as usize);
            std::ptr::write(end, value);
            self.len += 1;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(std::ptr::read(self.get_unchecked(self.len as usize)))
            }
        }
    }
}

impl<T> Deref for RageVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr as _, self.len as usize)
        }
    }
}

impl<T> DerefMut for RageVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.len as usize)
        }
    }
}

impl<T> Drop for RageVec<T> {
    fn drop(&mut self) {
        unsafe {
            self.capacity = 0;
            self.len = 0;
            if !self.ptr.is_null() {
                SysAlloc::global().free(self.ptr);
                self.ptr = std::ptr::null_mut();
            }
        }
    }
}

#[repr(transparent)]
pub struct ChainedBox<T> {
    inner: Box<T>
}

impl<T> Deref for ChainedBox<Chained<T>> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner.value
    }
}

#[repr(C)]
pub struct Chained<T> {
    value: T,
    next: Option<ChainedBox<Chained<T>>>
}

impl<T> ChainedBox<Chained<T>> {
    pub fn iter(&self) -> ChainedIter<T> {
        ChainedIter {
            current: Some(self)
        }
    }
}

pub struct ChainedIter<'a, T> {
    current: Option<&'a ChainedBox<Chained<T>>>
}

impl<'a, T> Iterator for ChainedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current.take() {
            self.current = current.inner.next.as_ref();
            Some(&current.inner.value)
        } else {
            None
        }
    }
}