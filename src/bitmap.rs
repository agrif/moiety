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
