use crate::future::*;

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

pub trait Display {
    // display-side bitmap
    type Bitmap;
    type Error: failure::Fail;

    fn transfer<'a>(
        &'a self,
        src: &'a Bitmap,
    ) -> Fut<'a, Result<Self::Bitmap, Self::Error>>;
    fn draw(
        &mut self,
        src: &Self::Bitmap,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    );
    fn flip(&mut self);
}
