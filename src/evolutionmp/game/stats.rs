use crate::invoke;
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
    fn read(hash: Hash, default: Self) -> Option<Self> {
        let mut result = 0;
        if invoke!(bool, 0x767FBC2AC802EF3D, hash, &mut result, default) {
            Some(result)
        } else {
            None
        }
    }

    fn write(&self, hash: Hash, save: bool) -> bool {
        invoke!(bool, 0xB3271D7AB655B441, hash, *self, save)
    }
}

impl StatValue for f32 {
    fn read(hash: Hash, default: Self) -> Option<Self> {
        let mut result = 0.0;
        if invoke!(bool, 0xD7AE6C9C9C6AC54C, hash, &mut result, default) {
            Some(result)
        } else {
            None
        }
    }

    fn write(&self, hash: Hash, save: bool) -> bool {
        invoke!(bool, 0x11B5E6D2AE73F48E, hash, *self, save)
    }
}

impl StatValue for bool {
    fn read(hash: Hash, default: Self) -> Option<Self> {
        let mut result = false;
        if invoke!(bool, 0x11B5E6D2AE73F48E, hash, &mut result, default) {
            Some(result)
        } else {
            None
        }
    }

    fn write(&self, hash: Hash, save: bool) -> bool {
        invoke!(bool, 0x4B33C4243DE0C432, hash, *self, save)
    }
}