use crate::game::Hash;
use std::num::Wrapping;

pub fn joaat<S>(s: S) -> Hash where S: Into<String> {
    let s = s.into();
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