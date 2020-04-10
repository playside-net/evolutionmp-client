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