use crate::native::pool::Handleable;
use crate::game::Handle;

pub struct Checkpoint {
    handle: Handle
}

crate::impl_handle!(Checkpoint);