use crate::game::Handle;
use crate::native::pool::Handleable;

pub struct Pickup {
    handle: Handle
}

crate::impl_handle!(Pickup);