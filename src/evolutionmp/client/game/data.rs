use crate::invoke;
use cgmath::Vector3;
use std::marker::PhantomData;

pub trait DataValue {
    fn push_to_array(self, array: &mut Array<Self>) where Self: Sized;
    fn get_from_array(array: &Array<Self>, index: u32) -> Self where Self: Sized;
    fn push_to_object(self, object: &mut Object, key: &str) where Self: Sized;
    fn get_from_object(object: &Object, key: &str) -> Self where Self: Sized;
    fn get_type() -> DataType;
}

pub enum DataType {
    Boolean = 1,
    Integer = 2,
    Float = 3,
    String = 4,
    Vector3 = 5,
    Object = 6,
    Array = 7
}

pub struct Array<V> {
    handle: u64,
    _ty: PhantomData<V>
}

impl<V> Array<V> {
    pub fn size(&self) -> usize {
        invoke!(u32, 0x065DB281590CEA2D, self.handle) as usize
    }

    pub fn get_type_at(&self, index: usize) -> Option<DataType> {
        if index < self.size() {
            let ty = invoke!(u32, 0x3A0014ADB172A3C5, self.handle, index as u32);
            Some(unsafe { std::mem::transmute(ty as u8) })
        } else {
            None
        }
    }
}

impl<V> Array<V> where V: DataValue {
    pub fn push(&mut self, value: V) {
        value.push_to_array(self)
    }

    pub fn get(&self, index: usize) -> Option<V> {
        if index < self.size() {
            Some(V::get_from_array(self, index as u32))
        } else {
            None
        }
    }

    pub fn get_type() -> DataType {
        V::get_type()
    }
}

impl Array<Object> {
    pub fn push<F>(&mut self, mut closure: F) where F: FnMut(&mut Object) {
        let mut object = Object {
            handle: invoke!(u64, 0x6889498B3E19C797)
        };
        closure(&mut object);
    }

    pub fn get(&self, index: usize) -> Option<Object> {
        if index < self.size() {
            Some(Object {
                handle: invoke!(u64, 0x8B5FADCC4E3A145F, self.handle)
            })
        } else {
            None
        }
    }

    pub fn get_type() -> DataType {
        DataType::Object
    }
}

pub struct Object {
    handle: u64
}

impl Object {
    pub fn set<K, V>(&mut self, key: K, value: V) where K: AsRef<str>, V: DataValue {
        value.push_to_object(self, key.as_ref())
    }

    pub fn set_array<K, V, F>(&mut self, key: K, mut closure: F) where K: AsRef<str>, V: DataValue, F: FnMut(&mut Array<V>) {
        let mut array = Array {
            handle: invoke!(u64, 0x5B11728527CA6E5F, key.as_ref()),
            _ty: PhantomData
        };
        closure(&mut array);
    }

    pub fn set_object<K, F>(&mut self, key: K, mut closure: F) where K: AsRef<str>, F: FnMut(&mut Object) {
        let mut object = Object {
            handle: invoke!(u64, 0xA358F56F10732EE1, key.as_ref())
        };
        closure(&mut object);
    }

    pub fn get<K, V>(&self, key: K) -> V where K: AsRef<str>, V: DataValue {
        V::get_from_object(self, key.as_ref())
    }

    pub fn get_array<K, V>(&self, key: K) -> Array<V> where K: AsRef<str>, V: DataValue {
        Array {
            handle: invoke!(u64, 0x7A983AA9DA2659ED, key.as_ref()),
            _ty: PhantomData
        }
    }

    pub fn get_object<K>(&self, key: K) -> Object where K: AsRef<str> {
        Object {
            handle: invoke!(u64, 0xB6B9DDC412FCEEE2, key.as_ref())
        }
    }
}

macro_rules! impl_data_value {
    ($ty:ty,$data_ty:ident,$arr_push:literal,$arr_get:literal,$obj_push:literal,$obj_get:literal) => {
        impl DataValue for $ty {
            fn push_to_array(self, array: &mut Array<Self>) {
                invoke!((), $arr_push, array.handle, self)
            }

            fn get_from_array(array: &Array<Self>, index: u32) -> Self {
                invoke!(Self, $arr_get, array.handle, index)
            }

            fn push_to_object(self, object: &mut Object, key: &str) {
                invoke!((), $obj_push, object.handle, key, self)
            }

            fn get_from_object(object: &Object, key: &str) -> Self {
                invoke!(Self, $obj_get, object.handle, key)
            }

            fn get_type() -> DataType {
                DataType::$data_ty
            }
        }
    };
}

impl_data_value!(bool, Boolean, 0xF8B0F5A43E928C76, 0x50C1B2874E50C114, 0x35124302A556A325, 0x1186940ED72FFEEC);
impl_data_value!(f32, Float, 0x57A995FD75D37F56, 0xC0C527B525D7CFB5, 0xC27E1CC2D795105E, 0x06610343E73B9727);
impl_data_value!(i32, Integer, 0xCABDB751D86FE93B, 0x3E5AE19425CD74BE, 0xE7E035450A7948D5, 0x78F06F6B1FB5A80C);
impl_data_value!(u32, Integer, 0xCABDB751D86FE93B, 0x3E5AE19425CD74BE, 0xE7E035450A7948D5, 0x78F06F6B1FB5A80C);
impl_data_value!(&str, String, 0x2F0661C155AEEEAA, 0xD3F2FFEB8D836F52, 0x8FF3847DADD8E30C, 0x3D2FD9E763B24472);
impl_data_value!(Vector3<f32>, Vector3, 0x407F8D034F70F0C2, 0x8D2064E5B64A628A, 0x4CD49B76338C7DEE, 0x46CD3CB66E0825CC);