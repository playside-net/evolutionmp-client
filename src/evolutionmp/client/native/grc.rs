use std::ops::Deref;
use std::os::raw::c_char;

use crate::bind_field_ip;
use crate::pattern::RageBox;

bind_field_ip!(TEXTURE_FACTORY, "84 DB 48 0F 45 C2 48 89 05", 9, TextureFactory);
bind_field_ip!(MISSING_TEXTURE, "45 33 C0 48 8B CF FF 50 20 48 8D 15", 22, Texture);

enum D3D11Resource {}

enum D3D11ShaderResourceView {}

pub(crate) fn hook() {
    info!("Hooking grc...");
    lazy_static::initialize(&TEXTURE_FACTORY);
    lazy_static::initialize(&MISSING_TEXTURE);
}

pub struct MappedTexture<'a> {
    texture: &'a mut Texture,
    locked: LockedTexture,
}

impl<'a> Drop for MappedTexture<'a> {
    fn drop(&mut self) {
        (self.texture.v_table.unmap)(self.texture, &mut self.locked)
    }
}

impl<'a> Deref for MappedTexture<'a> {
    type Target = LockedTexture;

    fn deref(&self) -> &Self::Target {
        &self.locked
    }
}

#[repr(C)]
#[derive(Default)]
pub struct LockedTexture {
    pub level: i32,
    pub p_bits: usize,
    pub pitch: i32,
    pub pad: i32,
    pub width: i32,
    pub height: i32,
    pub format: i32,
    pub sub_level_count: i32,
}

impl LockedTexture {
    pub unsafe fn commit(&self, data: &[u8]) {
        let bits = std::slice::from_raw_parts_mut(self.p_bits as *mut u8, data.len());
        bits.copy_from_slice(data);
    }
}

#[repr(C)]
pub struct TextureReference {
    pub width: u16,
    pub height: u16,
    pub format: i32,
    pub ty: u8,
    pad: u8,
    pad2: u8,
    pub stride: u16,
    pub depth: u16,
    pub pixel_data: *mut u8,
    pub pad3: usize,
    pub next_mip_level: *mut TextureReference,
    pub next_major_level: *mut TextureReference,
}

#[repr(C)]
#[derive(Default)]
struct ManualTextureDef {
    is_staging: i32,
    pad: [u8; 20],
    is_render_target: i32,
    pad2: [u8; 8],
    array_size: i32,
    pad3: [u8; 16],
}

impl ManualTextureDef {
    pub fn new(staging: bool, render_target: bool, array_size: usize) -> Self {
        Self {
            is_staging: staging as i32,
            is_render_target: render_target as i32,
            array_size: array_size as i32,
            ..Default::default()
        }
    }
}

#[repr(C)]
pub struct Texture {
    v_table: RageBox<TextureVTable>,
    m_pad: [u8; 48],
    texture: *mut D3D11Resource,
    m_pad2: [u8; 56],
    srv: *mut D3D11ShaderResourceView,
}

#[repr(C)]
struct TextureVTable {
    destructor: extern fn(this: *mut Texture),
    m_4: extern fn(this: *mut Texture) -> bool,
    m_8: extern fn(this: *mut Texture) -> i32,
    m_c: extern fn(this: *mut Texture),
    m_10: extern fn(this: *mut Texture) -> i32,
    get_width: extern fn(this: *mut Texture) -> u16,
    get_height: extern fn(this: *mut Texture) -> u16,
    get_depth: extern fn(this: *mut Texture) -> u16,
    get_levels: extern fn(this: *mut Texture) -> u8,
    m_24: extern fn(this: *mut Texture),
    m_28: extern fn(this: *mut Texture) -> bool,
    m_2c: extern fn(this: *mut Texture, unk: isize),
    m_30: extern fn(this: *mut Texture, unk: *mut ()),
    m_34: extern fn(this: *mut Texture, unk: *mut ()),
    m_unk: extern fn(this: *mut Texture),
    m_38: extern fn(this: *mut Texture) -> *mut Texture,
    m_3c: extern fn(this: *mut Texture) -> *mut Texture,
    m_40: extern fn(this: *mut Texture) -> bool,
    m_44: extern fn(this: *mut Texture) -> i32,
    m_48: extern fn(this: *mut Texture) -> i32,
    m_4c: extern fn(this: *mut Texture) -> i32,
    m_50: extern fn(this: *mut Texture) -> i32,
    m_54: extern fn(this: *mut Texture) -> i32,
    m_unk2: extern fn(this: *mut Texture) -> i32,
    m_unk3: extern fn(this: *mut Texture) -> i32,
    map: extern fn(this: *mut Texture, sub_level_count: i32, sub_level: i32, locked_texture: *mut LockedTexture, flags: u32) -> bool,
    unmap: extern fn(this: *mut Texture, locked_texture: *mut LockedTexture),
}

impl Texture {
    pub fn map(&mut self, sub_level_count: i32, sub_level: i32, flags: u32) -> Option<MappedTexture> {
        let mut locked = LockedTexture::default();
        if (self.v_table.map)(self, sub_level_count, sub_level, &mut locked, flags) {
            Some(MappedTexture {
                texture: self,
                locked,
            })
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct TextureFactory {
    v_table: RageBox<TextureFactoryVTable>
}

#[repr(C)]
struct TextureFactoryVTable {
    destructor: extern fn(this: *mut TextureFactory),
    unk_8: extern fn(this: *mut TextureFactory) -> *mut Texture,
    new_manual_texture: extern fn(this: *mut TextureFactory, w: u16, h: u16, format: i32, unk1: *mut (), unk2: bool, template: *const ()) -> *mut Texture,
    new_image: extern fn(this: *mut TextureFactory, tex: *mut (), template: *mut ()) -> *mut Texture,
    v0: extern fn(this: *mut TextureFactory),
    v1: extern fn(this: *mut TextureFactory),
    translate_format: extern fn(this: *mut TextureFactory, format: i32) -> i32,
    v2: extern fn(this: *mut TextureFactory),
    v3: extern fn(this: *mut TextureFactory),
    v4: extern fn(this: *mut TextureFactory),
    v5: extern fn(this: *mut TextureFactory),
    v6: extern fn(this: *mut TextureFactory),
    v7: extern fn(this: *mut TextureFactory),
    v8: extern fn(this: *mut TextureFactory),
    v9: extern fn(this: *mut TextureFactory),
    v10: extern fn(this: *mut TextureFactory),
    v11: extern fn(this: *mut TextureFactory),
    new_from_native: extern fn(this: *mut TextureFactory, name: *const c_char, resource: *mut (), a3: *mut ()),
}

impl TextureFactory {
    pub unsafe fn get_mut<'a>() -> &'a mut TextureFactory {
        TEXTURE_FACTORY.as_mut()
    }
}

