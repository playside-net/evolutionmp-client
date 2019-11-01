use std::num::Wrapping;

pub type Hash = u32;

pub fn joaat<S>(s: S) -> Hash where S: AsRef<str> {
    let s = s.as_ref();
    let mut hash = Wrapping(0u32);
    for c in s.chars() {
        hash += Wrapping(c.to_lowercase().next().unwrap() as u32);
        hash += (hash << 10);
        hash ^= (hash >> 6);
    }
    hash += (hash << 3);
    hash ^= (hash >> 11);
    hash += (hash << 15);
    hash.0
}

pub trait Hashable {
    fn joaat(&self) -> u32;
}

impl<'a> Hashable for &'a str {
    fn joaat(&self) -> u32 {
        crate::hash::joaat(self)
    }
}

impl Hashable for String {
    fn joaat(&self) -> u32 {
        crate::hash::joaat(self)
    }
}

impl Hashable for Hash {
    fn joaat(&self) -> u32 {
        *self
    }
}