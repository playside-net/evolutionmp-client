use crate::game::Handle;
use crate::native::pool::FromHandle;

pub struct Object {
    handle: Handle
}

impl FromHandle for Object {
    unsafe fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }
}