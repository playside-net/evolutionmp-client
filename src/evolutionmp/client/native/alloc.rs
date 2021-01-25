use std::ops::{Deref, DerefMut};
use std::alloc::{Layout};

#[repr(C)]
pub struct RageVec<T> {
    ptr: *mut T,
    len: u16,
    capacity: u16
}

impl<T> RageVec<T> {
    #[inline]
    pub fn empty() -> RageVec<T> {
        RageVec {
            ptr: std::ptr::NonNull::dangling().as_ptr(),
            len: 0,
            capacity: 0
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

    unsafe fn layout(capacity: usize) -> Layout {
        let align = std::mem::align_of::<T>();
        let size = std::mem::size_of::<T>() * capacity;
        Layout::from_size_align_unchecked(size, align)
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
#[repr(C)]
pub struct RageCollection<T> {
    ptr: *mut T,
    len: u16,
    capacity: u16
}

impl<T> RageCollection<T> {
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
}

pub struct RCIter<T> {
    start: *mut T,
    end: *mut T
}

#[repr(C)]
pub struct Chained<T> {
    pub value: T,
    next: Option<Box<Chained<T>>>
}

pub type ChainedBox<T> = Box<Chained<T>>;

impl<T> Chained<T> {
    pub fn iter(&self) -> ChainedIter<T> {
        ChainedIter {
            current: Some(self)
        }
    }
}

pub struct ChainedIter<'a, T> {
    current: Option<&'a Chained<T>>
}

impl<'a, T> Iterator for ChainedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current.take() {
            self.current = current.next.as_deref();
            Some(&current.value)
        } else {
            None
        }
    }
}