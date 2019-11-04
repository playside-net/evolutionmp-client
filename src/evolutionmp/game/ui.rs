use crate::{invoke, native};
use crate::game::Rgba;
use cgmath::Vector2;

pub const BASE_WIDTH: f32 = 1280.0;
pub const BASE_HEIGHT: f32 = 720.0;

pub fn show_loading_prompt(ty: LoadingPrompt, text: &str) {
    unsafe {
        crate::native::ui::set_loading_prompt_text_entry("STRING");
        crate::native::ui::push_string(text);
        crate::native::ui::show_loading_prompt(ty as u32);
    }
}

pub fn show_subtitle(text: &str, duration: i32, immediately: bool) {
    unsafe {
        use crate::invoke;
        invoke!((), 0xB87A37EEB7FAA67D, "STRING");
        crate::native::ui::push_string(text);
        invoke!((), 0x9D77056A530643F6, duration, immediately);
    }
}

pub fn draw_rect<P, S, C>(pos: P, size: S, color: C)
    where P: Into<Vector2<f32>>, S: Into<Vector2<f32>>, C: Into<Rgba>
{
    let pos = pos.into();
    let pos = Vector2::new(pos.x / BASE_WIDTH, pos.y / BASE_HEIGHT);
    let size = size.into();
    let size = Vector2::new(size.x / BASE_WIDTH, size.y / BASE_HEIGHT);

    unsafe { native::ui::draw_rect(pos + (size * 0.5), size, color.into()) }
}

pub fn draw_text<T, P, S>(text: T, pos: P, color: Rgba, font: Font, scale: S)
    where T: AsRef<str>, P: Into<Vector2<f32>>, S: Into<Vector2<f32>>
{
    let pos = pos.into();
    let pos = Vector2::new(pos.x / BASE_WIDTH, pos.y / BASE_HEIGHT);
    unsafe {
        native::ui::set_text_font(font as u32);
        native::ui::set_text_scale(scale.into());
        native::ui::set_text_color(color);
        native::ui::begin_text_command_draw("CELL_EMAIL_BCON");
        native::ui::push_string(text.as_ref());
        native::ui::end_text_command_draw(pos.into())
    }
}

pub fn get_text_width<T>(text: T, font: Font, scale: Vector2<f32>) -> f32 where T: AsRef<str> {
    unsafe {
        native::ui::set_text_font(font as u32);
        native::ui::set_text_scale(scale);
        native::ui::begin_text_command_width("CELL_EMAIL_BCON");
        native::ui::push_string(text.as_ref());
        native::ui::end_text_command_width(true)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Font {
    ChaletLondon,
    HouseScript,
    Monospace,
    ChaletComprimeCologne = 4,
    Pricedown = 7
}

pub enum LoadingPrompt {
    LoadingLeft,
    LoadingLeft2,
    LoadingLeft3,
    SavingLeft,
    LoadingRight
}

pub fn hide_loading_prompt() {
    unsafe {
        crate::native::ui::hide_loading_prompt();
    }
}

pub fn set_cursor_sprite(sprite: CursorSprite) {
    unsafe {
        crate::native::ui::set_cursor_sprite(sprite as u32)
    }
}

#[repr(C)]
pub enum CursorSprite {
    None,
    Normal,
    TransparentNormal,
    PreGrab,
    Grab,
    MiddleFinger,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    HorizontalExpand,
    Add,
    Remove
}

pub enum HudComponent {
    Main,
    WantedStars,
    WeaponIcon,
    Cash,
    MpCash,
    MpMessage,
    VehicleName,
    AreaName,
    Unused,
    StreetName,
    HelpText,
    FloatingHelpText1,
    FloatingHelpText2,
    CashChange,
    Reticle,
    SubtitleText,
    RadioStationsWheel,
    Saving,
    GameStreamUnused,
    WeaponWheel,
    WeaponWheelStats,
    DrugsPurse01,
    DrugsPurse02,
    DrugsPurse03,
    DrugsPurse04,
    MpTagCashFromBank,
    MpTagPackages,
    MpTagCuffKeys,
    MpTagDownloadData,
    MpTagIfPedFollowing,
    MpTagKeyCard,
    MpTagRandomObject,
    MpTagRemoteControl,
    MpTagCashFromSafe,
    MpTagWeaponsPackage,
    MpTagKeys,
    MpVehicle,
    MpVehicleHelicopter,
    MpVehiclePlane,
    PlayerSwitchAlert,
    MpRankBar,
    DirectorMode,
    ReplayController,
    ReplayMouse,
    ReplayHeader,
    ReplayOptions,
    ReplayHelpText,
    ReplayMiscText,
    ReplayTopLine,
    ReplayBottomLine,
    ReplayLeftBar,
    ReplayTimer
}