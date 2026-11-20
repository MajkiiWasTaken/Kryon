use gpui::{Hsla, rgb};

pub struct KryonTheme;

impl KryonTheme {
    pub const BACKGROUND: u32 = 0x111318;
    pub const SIDEBAR: u32 = 0x191D25;
    pub const SURFACE: u32 = 0x1B2029;
    pub const SURFACE_HOVER: u32 = 0x293241;
    pub const ACTIVE: u32 = 0x252D3A;
    pub const BORDER: u32 = 0x29303B;

    pub const TEXT: u32 = 0xE6EAF2;
    pub const TEXT_SECONDARY: u32 = 0xBBC4D2;
    pub const MUTED: u32 = 0x8E97A8;

    pub const ACCENT: u32 = 0x87AFFF;
    pub const FOLDER: u32 = 0xE4BA78;
    pub const SUCCESS: u32 = 0x92C7A0;
    pub const ERROR: u32 = 0xED929A;

    pub fn color(value: u32) -> Hsla {
        rgb(value).into()
    }
}
