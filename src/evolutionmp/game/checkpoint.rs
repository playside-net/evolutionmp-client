use crate::native::pool::FromHandle;
use crate::game::Handle;

pub struct Checkpoint {
    handle: Handle
}

impl FromHandle for Checkpoint {
    fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }
}