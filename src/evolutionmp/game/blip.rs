use cgmath::{Vector2, Vector3};

use crate::game::entity::Entity;
use crate::game::Handle;
use crate::invoke;

pub fn get_pool() -> BlipIterator {
    BlipIterator::new()
}

#[derive(Debug)]
pub struct Blip {
    handle: Handle
}

crate::impl_handle!(Blip);

pub enum BlipName<'a, 'b, 'c> {
    Localized(&'a str, &'b [&'c str]),
    Generic(&'a str),
}

impl Blip {
    pub fn player() -> Blip {
        invoke!(Blip, 0xDCD4EC3F419D02FA)
    }

    pub fn new_for_area(pos: Vector3<f32>, size: Vector2<f32>) -> Blip {
        invoke!(Blip, 0xCE5D0E5E315DB238, pos, size)
    }

    pub fn new_for_pos(pos: Vector3<f32>) -> Blip {
        invoke!(Blip, 0x5A039BB0BCA604B6, pos)
    }

    pub fn new_for_entity(entity: &dyn Entity) -> Blip {
        invoke!(Blip, 0x5CDE92C702A8FCE7, entity.get_handle())
    }

    pub fn new_for_radius(pos: Vector3<f32>, radius: f32) -> Blip {
        invoke!(Blip, 0x46818D79B1F7499A, pos, radius)
    }

    pub fn from_entity(entity: &dyn Entity) -> Option<Blip> {
        invoke!(Option<Blip>, 0xBC8DBDCA2436F7E8, entity.get_handle())
    }

    pub fn set_rotation(&self, rotation: f32) {
        invoke!((), 0xF87683CDF73C3F6E, self.handle, super::system::ceil(rotation))
    }

    pub fn set_alpha(&self, alpha: u8) {
        invoke!((), 0x45FF974EEE1C8734, self.handle, alpha as u32)
    }

    pub fn set_color(&self, color: u32) {
        invoke!((), 0x03D7FB09E75D6B7E, self.handle, color)
    }

    pub fn set_secondary_color(&self, color: Vector3<f32>) {
        invoke!((), 0x14892474891E09EB, color)
    }

    pub fn set_name(&self, name: BlipName) {
        match name {
            BlipName::Localized(format, args) => {
                invoke!((), 0xF9113A30DE5C6670, format);
                for arg in args {
                    super::ui::push_string(arg);
                }
            }
            BlipName::Generic(name) => {
                invoke!((), 0xF9113A30DE5C6670, "STRING");
                super::ui::push_string(name);
            }
        }
        invoke!((), 0xBC38B49BCB83BC9B, self.handle)
    }

    pub fn set_friendly(&self, friendly: bool) {
        invoke!((), 0x6F6F290102C02AB4, self.handle, friendly)
    }

    pub fn set_short_range(&self, short_range: bool) {
        invoke!((), 0xBE8BE4FE60E27B72, self.handle, short_range)
    }

    pub fn set_bright(&self, bright: bool) {
        invoke!((), 0xB203913733F27884, self.handle, bright)
    }

    pub fn set_category(&self, category: u8) {
        invoke!((), 0x234CDD44D996FD9A, self.handle, category as u32)
    }

    pub fn set_position(&self, pos: Vector3<f32>) {
        invoke!((), 0xAE2AF67E9D9AF65D, self.handle, pos)
    }

    pub fn set_display_mode(&self, mode: u8) {
        invoke!((), 0x9029B2F3DA924928, self.handle, mode as u32)
    }

    pub fn set_fade(&self, opacity: u32, duration: u32) {
        invoke!((), 0x2AEE8F8390D2298C, self.handle, opacity, duration)
    }

    pub fn set_flash_interval(&self, interval: u32) {
        invoke!((), 0xAA51DB313C010A7E, self.handle, interval)
    }

    pub fn set_flash_duration(&self, duration: u32) {
        invoke!((), 0xD3CD6FD297AE87CC, self.handle, duration)
    }

    pub fn set_flashes(&self, flashes: bool) {
        invoke!((), 0xB14552383D39CE3E, self.handle, flashes)
    }

    pub fn set_flashes_alternate(&self, flashes: bool) {
        invoke!((), 0x2E8D9498C56DD0D1, self.handle, flashes)
    }

    pub fn set_hidden_on_legend(&self, hidden: bool) {
        invoke!((), 0x54318C915D27E4CE, self.handle, hidden)
    }

    pub fn set_high_detail(&self, high_detail: bool) {
        invoke!((), 0xE2590BC29220CEBB, self.handle, high_detail)
    }

    pub fn set_priority(&self, priority: u32) {
        invoke!((), 0xAE9FC9EF6A9FAC79, self.handle, priority)
    }

    pub fn set_route(&self, route: bool) {
        invoke!((), 0x4F7D8A9BFB0B43E9, self.handle, route)
    }

    pub fn set_route_color(&self, color: u32) {
        invoke!((), 0x837155CD2F63DA09, self.handle, color)
    }

    pub fn set_scale(&self, scale: f32) {
        invoke!((), 0xD38744167B2FA257, self.handle, scale)
    }

    pub fn set_show_cone(&self, show_cone: bool) {
        invoke!((), 0x13127EC3665E8EE1, self.handle, show_cone)
    }

    pub fn set_shrink(&self, shrink: bool) {
        invoke!((), 0x2B6D467DAB714E8D, self.handle, shrink)
    }

    pub fn set_sprite(&self, sprite: u32) {
        invoke!((), 0xDF735600A4696DAF, self.handle, sprite)
    }

    pub fn set_squared_rotation(&self, rotation: f32) {
        invoke!((), 0xA8B6AFDAC320AC87, self.handle, rotation)
    }

    /// Color can be changed via set_secondary_color
    pub fn set_show_left_half_circle(&self, show: bool) {
        invoke!((), 0xDCFB5D4DB8BF367E, self.handle, show)
    }

    pub fn set_show_right_half_circle(&self, show: bool) {
        invoke!((), 0x23C3EB807312F01A, self.handle, show)
    }

    pub fn set_show_heading_indicator(&self, show: bool) {
        invoke!((), 0x5FBCA48327B914DF, self.handle, show)
    }

    pub fn set_show_height(&self, show: bool) {
        invoke!((), 0x75A16C3DA34F1245, self.handle, show)
    }

    pub fn set_show_number(&self, show: bool) {
        invoke!((), 0xA3C0B359DCB848B6, self.handle, show)
    }

    /// Overrides set_show_left_half_circle & set_show_right_half_circle
    /// Color can be changed via set_secondary_color
    pub fn set_show_outline(&self, show: bool) {
        invoke!((), 0xB81656BC81FE24D1, self.handle, show)
    }

    pub fn set_show_green_check_mark(&self, show: bool) {
        invoke!((), 0x74513EA3E505181E, self.handle, show)
    }

    pub fn get_alpha(&self) -> u8 {
        invoke!(u32, 0x970F608F0EE6C885, self.handle) as u8
    }

    pub fn get_color(&self) -> u32 {
        invoke!(u32, 0xDF729E8D20CF7327, self.handle)
    }

    pub fn get_hud_color(&self) -> u32 {
        invoke!(u32, 0x729B5F1EFBC0AAEE, self.handle)
    }

    pub fn get_position(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0x586AFE3FF72D996E, self.handle)
    }

    pub fn get_sprite(&self) -> u32 {
        invoke!(u32, 0x1FC877464A04FC4F, self.handle)
    }

    pub fn get_type(&self) -> u32 {
        invoke!(u32, 0xBE9B0959FFD0779B, self.handle)
    }

    pub fn is_flashing(&self) -> bool {
        invoke!(bool, 0xA5E41FD83AD6CEF0, self.handle)
    }

    pub fn is_on_minimap(&self) -> bool {
        invoke!(bool, 0xE41CA53051197A27, self.handle)
    }

    pub fn is_short_range(&self) -> bool {
        invoke!(bool, 0xDA5F8727EB75B926, self.handle)
    }

    pub fn zoom_radar(&self, zoom: f32) {
        invoke!((), 0xF98E4B3E56AFC7B1, self.handle, zoom)
    }

    pub fn hide_number(&self) {
        invoke!((), 0x532CFF637EF80148, self.handle)
    }

    pub fn pulse(&self) {
        invoke!((), 0x742D6FD43115AF73, self.handle)
    }

    pub fn exists(&self) -> bool {
        invoke!(bool, 0xA6DB27D19ECBB7DA, self.handle)
    }

    pub fn has_gps_route(&self) -> bool {
        invoke!(bool, 0xDD2238F57B977751, self.handle)
    }

    pub fn delete(&mut self) {
        invoke!((), 0x86A652570E5F25DD, &mut self.handle)
    }
}

pub struct BlipIterator {
    handle: Handle,
    first: bool,
}

impl BlipIterator {
    pub fn new() -> BlipIterator {
        BlipIterator {
            handle: invoke!(Handle, 0x186E5D252FA50E7D),
            first: true,
        }
    }
}

impl Iterator for BlipIterator {
    type Item = Blip;

    fn next(&mut self) -> Option<Blip> {
        if self.first {
            self.first = false;
            invoke!(Option<Blip>, 0x1BEDE233E6CD2A1F, self.handle)
        } else {
            invoke!(Option<Blip>, 0x14F96AA50D6FBEA7, self.handle)
        }
    }
}