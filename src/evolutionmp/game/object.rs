use crate::game::Handle;
use crate::native::pool::Handleable;

pub struct Object {
    handle: Handle
}

crate::impl_handle!(Object);