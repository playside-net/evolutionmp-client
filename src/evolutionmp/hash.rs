use std::num::Wrapping;

use serde_derive::{Serialize, Deserialize};

#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Hash(pub u32);

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = self.0 as i32;
        if let Some(name) = crate::native::OBJECT_HASHES.get(&raw) {
            f.write_str(name)
        } else {
            f.write_fmt(format_args!("0x{:08X}", self.0))
        }
    }
}

impl AsRef<dyn Hashable> for Hash {
    fn as_ref(&self) -> &(dyn Hashable + 'static) {
        self
    }
}

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

//Case-sensitive version of joaat
pub fn joaat_cs<S>(s: S) -> Hash where S: AsRef<str> {
    let s = s.as_ref();
    let mut hash = Wrapping(0u32);
    for c in s.chars() {
        hash += Wrapping(c as u32);
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

    fn to_string(&self) -> String {
        format!("{}", self.joaat())
    }
}

impl Hashable for Hash {
    fn joaat(&self) -> Hash {
        *self
    }

    fn to_string(&self) -> String {
        format!("0x{:08X}", self.0)
    }
}

impl Hashable for &str {
    fn joaat(&self) -> Hash {
        crate::hash::joaat(self)
    }

    fn to_string(&self) -> String {
        String::from(*self)
    }
}

impl<'a, H> Hashable for &'a H where H: Hashable {
    fn joaat(&self) -> Hash {
        (*self).joaat()
    }

    fn to_string(&self) -> String {
        (*self).to_string()
    }
}