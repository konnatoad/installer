use egui::Color32;

pub const BG: Color32 = Color32::from_gray(6);
pub const SURFACE: Color32 = Color32::from_gray(16);
pub const SURFACE2: Color32 = Color32::from_gray(22);
pub const SURFACE3: Color32 = Color32::from_gray(30);
pub const SURFACE4: Color32 = Color32::from_gray(48);
pub const BORDER: Color32 = Color32::from_gray(40);
pub const ACCENT: Color32 = Color32::from_rgb(60, 130, 220);
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(84, 156, 240);
pub const ACCENT2: Color32 = Color32::from_rgb(0xbb, 0x9a, 0xf7);
pub const TEXT: Color32 = Color32::from_gray(225);
pub const TEXT_DIM: Color32 = Color32::from_gray(145);
pub const TEXT_MUTED: Color32 = Color32::from_gray(100);
pub const SUCCESS: Color32 = Color32::from_rgb(0x9e, 0xce, 0x6a);
pub const ERROR_TEXT: Color32 = Color32::from_rgb(0xf7, 0x76, 0x8e);

pub fn accent_fill(alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(60, 130, 220, alpha)
}

pub fn white_wash(alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, alpha)
}
