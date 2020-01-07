use crate::game::Handle;
use crate::native::pool::FromHandle;

pub struct Pickup {
    handle: Handle
}

impl FromHandle for Pickup {
    unsafe fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }
}