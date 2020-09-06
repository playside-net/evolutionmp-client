use crate::invoke;
use crate::game::{Handle, Rgba};
use crate::game::streaming::Texture;

#[derive(Debug)]
pub struct Notification {
    handle: Handle
}

impl Notification {
    pub fn delete(&self) {
        invoke!((), 0xBE4390CB40B3E627, self.handle)
    }
}

crate::impl_handle!(Notification);

pub struct NotificationFlash {
    color: Rgba,
    count: u32
}

impl NotificationFlash {
    pub fn new(color: Rgba, count: u32) -> NotificationFlash {
        NotificationFlash {
            color, count
        }
    }

    fn apply(&self) {
        invoke!((), 0x17430B918701C342, self.color);
        invoke!((), 0x17AD8C9706BDD88A, self.count);
    }
}

pub struct NotificationColor {
    foreground: Rgba,
    background: Rgba
}

impl NotificationColor {
    pub fn new(foreground: Rgba, background: Rgba) -> NotificationColor {
        NotificationColor {
            foreground, background
        }
    }

    fn apply(&self) {
        invoke!((), 0x39BBF623FC803EAC, self.foreground);
        invoke!((), 0x92F0DA1E27DB96DC, self.background);
    }
}

pub fn send_notification(text: &str, color: Option<NotificationColor>, flash: Option<NotificationFlash>, log: bool) -> Notification {
    invoke!((), 0x202709F4C58A0424, "STRING");
    if let Some(color) = &color {
        color.apply();
    }
    if let Some(flash) = &flash {
        flash.apply();
    }
    super::push_string(text);
    invoke!(Notification, 0x2ED7843F8F801023, flash.is_some(), log)
}

pub fn send_notification_award(text: &str, rp_bonus: u32, color_overlay: u32, texture: Texture) -> Notification {
    invoke!((), 0x202709F4C58A0424, "STRING");
    super::push_string(text);
    invoke!(Notification, 0xAA295B6F28BD587D, texture, rp_bonus, color_overlay, "FM_GEN_UNLOCK")
}