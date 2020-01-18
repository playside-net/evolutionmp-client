use crate::native::pool::Handleable;
use crate::game::Handle;

pub struct Checkpoint {
    handle: Handle
}

impl Handleable for Checkpoint {
    fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }

    fn get_handle(&self) -> Handle {
        self.handle
    }
}