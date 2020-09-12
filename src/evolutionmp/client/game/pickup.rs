use crate::game::Handle;
use crate::client::native::pool::CPickup;

pub struct Pickup {
    handle: Handle
}

crate::impl_native!(Pickup, CPickup);