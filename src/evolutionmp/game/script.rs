use crate::native;

pub fn shutdown_loading_screen() {
    unsafe { native::script::shutdown_loading_screen() }
}