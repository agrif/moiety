#[derive(Debug, Clone)]
pub struct Bitmap {
    pub width: u16,
    pub height: u16,
    pub palette: Option<PaletteBitmap>,
    pub data: Vec<palette::Srgb<u8>>,
}

#[derive(Debug, Clone)]
pub struct PaletteBitmap {
    pub palette: Vec<palette::Srgb<u8>>,
    pub image: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub width: u16,
    pub height: u16,
    pub hotspot: (u16, u16),
    pub data: Vec<palette::Srgba<u8>>,
}
