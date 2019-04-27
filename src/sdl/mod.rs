use crate::{
    future::*,
    game::{
        Event,
        EventPump,
        Game,
    },
};

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

pub fn convert_sdl_event(ev: sdl2::event::Event) -> Option<Event> {
    match ev {
        sdl2::event::Event::Quit { .. } => Some(Event::Quit),
        sdl2::event::Event::MouseButtonDown {
            mouse_btn: sdl2::mouse::MouseButton::Left,
            x,
            y,
            ..
        } => Some(Event::MouseDown(x, y)),
        sdl2::event::Event::MouseButtonUp {
            mouse_btn: sdl2::mouse::MouseButton::Left,
            x,
            y,
            ..
        } => Some(Event::MouseUp(x, y)),
        _ => None,
    }
}

pub async fn run_sdl<G>(
    title: &str,
    width: u32,
    height: u32,
    game: G,
) -> Result<(), G::Error>
where
    G: Game<'static, Display>,
{
    let ctx = sdl2::init().unwrap();
    let display = Display::new(&ctx, title, width, height).unwrap();

    let pump = EventPump::new();
    let mut sender = pump.sender();

    let mut gamefut = game.start(pump, display);

    // FIXME proper error handling
    'mainloop: loop {
        if let Some(sdl_event) = ctx.event_pump().unwrap().poll_event() {
            if let Some(event) = convert_sdl_event(sdl_event) {
                if event == Event::Quit {
                    await!(sender.send(event));
                    break 'mainloop;
                } else {
                    await!(sender.send(event));
                }
            } else {
                await!(sender.send(Event::Idle));
            }
        } else {
            await!(sender.send(Event::Idle));
        }
        // give the game thread some time
        match futures::poll!(&mut gamefut) {
            std::task::Poll::Pending => (),
            std::task::Poll::Ready(result) => return Ok(result?),
        }
    }

    Ok(await!(gamefut)?)
}

pub struct Display {
    canvas: sdl2::render::WindowCanvas,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

impl Display {
    pub fn new(
        ctx: &sdl2::Sdl,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Self, SdlError> {
        let video = ctx.video()?;
        let window = video
            .window(title, width, height)
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().software().build()?;
        let texture_creator = canvas.texture_creator();

        Ok(Display {
            canvas,
            texture_creator,
        })
    }
}

impl crate::display::Display for Display {
    type Bitmap = sdl2::render::Texture;
    type Error = SdlError;

    fn transfer<'a>(
        &'a self,
        src: &'a crate::display::Bitmap,
    ) -> Fut<'a, Result<Self::Bitmap, Self::Error>> {
        fut!({
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
        })
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
