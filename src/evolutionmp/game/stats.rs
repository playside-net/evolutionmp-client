use crate::hash::{Hashable, Hash};
use std::marker::PhantomData;

pub struct Stat<V> where V: StatValue {
    hash: Hash,
    _ty: PhantomData<V>
}

impl<V> Stat<V> where V: StatValue {
    pub fn new<H>(hash: H) -> Stat<V> where H: Hashable {
        Stat {
            hash: hash.joaat(),
            _ty: PhantomData
        }
    }

    pub fn get(&self, default: V) -> Option<V> {
        V::read(self.hash, default)
    }

    pub fn set(&self, value: V, save: bool) -> bool {
        value.write(self.hash, save)
    }
}

pub trait StatValue {
    fn read(hash: Hash, default: Self) -> Option<Self> where Self: Sized;
    fn write(&self, hash: Hash, save: bool) -> bool;
}

impl StatValue for i32 {
    fn read(hash: Hash, default: i32) -> Option<Self> {
        let mut result = &mut 0;
        if crate::native::stats::get_int(hash, result, -1) {
            Some(*result)
        } else {
            None
        }
    }

    fn write(&self, hash: Hash, save: bool) -> bool {
        crate::native::stats::set_int(hash, *self, save)
    }
}