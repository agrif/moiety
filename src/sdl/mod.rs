#[derive(Fail, Debug)]
pub enum SdlError {
    #[fail(display = "{}", _0)]
    WindowBuildError(#[cause] sdl2::video::WindowBuildError),
    #[fail(display = "{}", _0)]
    TextureValueError(#[cause] sdl2::render::TextureValueError),
    #[fail(display = "{}", _0)]
    UpdateTextureError(#[cause] sdl2::render::UpdateTextureError),
    #[fail(display = "{}", _0)]
    IntegerOverflows(&'static str, u32),
    #[fail(display = "{}", _0)]
    Other(String),
}

impl std::convert::From<sdl2::video::WindowBuildError> for SdlError {
    fn from(other: sdl2::video::WindowBuildError) -> Self {
        SdlError::WindowBuildError(other)
    }
}

impl std::convert::From<sdl2::render::TextureValueError> for SdlError {
    fn from(other: sdl2::render::TextureValueError) -> Self {
        SdlError::TextureValueError(other)
    }
}

impl std::convert::From<sdl2::render::UpdateTextureError> for SdlError {
    fn from(other: sdl2::render::UpdateTextureError) -> Self {
        SdlError::UpdateTextureError(other)
    }
}

impl std::convert::From<sdl2::IntegerOrSdlError> for SdlError {
    fn from(other: sdl2::IntegerOrSdlError) -> Self {
        match other {
            sdl2::IntegerOrSdlError::IntegerOverflows(a, b) => {
                SdlError::IntegerOverflows(a, b)
            },
            sdl2::IntegerOrSdlError::SdlError(s) => SdlError::Other(s),
        }
    }
}

impl std::convert::From<String> for SdlError {
    fn from(other: String) -> Self { SdlError::Other(other) }
}

pub struct Display {
    ctx: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    canvas: sdl2::render::WindowCanvas,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

impl Display {
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Display, SdlError> {
        let ctx = sdl2::init()?;
        let video = ctx.video()?;
        let window = video
            .window(title, width, height)
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().software().build()?;
        let texture_creator = canvas.texture_creator();

        Ok(Display {
            ctx,
            video,
            canvas,
            texture_creator,
        })
    }

    pub fn events(&self) -> Result<bool, SdlError> {
        let event = self.ctx.event_pump()?.wait_event();
        match event {
            sdl2::event::Event::Quit { .. } => Ok(false),
            sdl2::event::Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Escape),
                ..
            } => Ok(false),
            _ => Ok(true),
        }
    }
}

impl crate::display::Display for Display {
    type Bitmap = sdl2::render::Texture;
    type Error = SdlError;

    fn transfer(
        &self,
        src: &crate::display::Bitmap,
    ) -> Result<Self::Bitmap, Self::Error> {
        let mut tex = self.texture_creator.create_texture_static(
            sdl2::pixels::PixelFormatEnum::RGB24,
            src.width as u32,
            src.height as u32,
        )?;
        tex.update(
            None,
            palette::Pixel::into_raw_slice(&src.data),
            src.width as usize * 3,
        )?;
        Ok(tex)
    }

    fn draw(
        &mut self,
        src: &Self::Bitmap,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) {
        self.canvas
            .copy(
                src,
                None,
                sdl2::rect::Rect::new(
                    left,
                    top,
                    (right - left) as u32,
                    (bottom - top) as u32,
                ),
            )
            .unwrap();
    }

    fn flip(&mut self) { self.canvas.present(); }
}
