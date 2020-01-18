use crate::game::Handle;
use crate::native::pool::Handleable;

pub struct Pickup {
    handle: Handle
}

impl Handleable for Pickup {
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