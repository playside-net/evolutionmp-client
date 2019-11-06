use std::num::Wrapping;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Hash(pub u32);

pub fn joaat<S>(s: S) -> Hash where S: AsRef<str> {
    let s = s.as_ref();
    let mut hash = Wrapping(0u32);
    for c in s.chars() {
        hash += Wrapping(c.to_lowercase().next().unwrap() as u32);
        hash += hash << 10;
        hash ^= hash >> 6;
    }
    hash += hash << 3;
    hash ^= hash >> 11;
    hash += hash << 15;
    Hash(hash.0)
}

pub trait Hashable {
    fn joaat(&self) -> Hash;
}

impl Hashable for Hash {
    fn joaat(&self) -> Hash {
        *self
    }
}

impl<S> Hashable for S where S: AsRef<str> {
    fn joaat(&self) -> Hash {
        crate::hash::joaat(self)
    }
}