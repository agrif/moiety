use crate::{Bitmap, Context, Event, Game, GameRunner};

use anyhow::{anyhow, Result};
use palette::Pixel;
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

pub struct Sdl {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl Sdl {
    pub async fn run<G>(mut game: G) -> Result<()>
    where
        G: Game,
    {
        let ctx = sdl2::init().map_err(|e| anyhow!(e))?;
        let video = ctx.video().map_err(|e| anyhow!(e))?;
        let window_size = game.window_size();
        let window = video
            .window(game.window_title(), window_size.0, window_size.1)
            .position_centered()
            .allow_highdpi()
            .build()
            .map_err(|e| anyhow!(e))?;

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| anyhow!(e))?;

        canvas.set_logical_size(window_size.0, window_size.1)?;
        let mut event_pump = ctx.event_pump().map_err(|e| anyhow!(e))?;

        let mut gamectx = Context::new(&game, Sdl { canvas });

        game.start(&mut gamectx).await?;

        'running: loop {
            for event in event_pump.poll_iter() {
                let mevent = match event {
                    SdlEvent::Quit { .. } => Event::Exit,
                    SdlEvent::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => Event::Exit,
                    SdlEvent::MouseButtonDown {
                        mouse_btn: MouseButton::Left,
                        x,
                        y,
                        ..
                    } => Event::MouseDown(x, y),
                    _ => Event::Idle,
                };

                if !game.handle_event(&mut gamectx, mevent).await? {
                    break 'running;
                }
            }

            smol::Timer::new(std::time::Duration::new(0, 1_000_000_000u32 / 60)).await;
        }
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl GameRunner for Sdl {
    async fn output(&mut self, frame: &Bitmap) -> Result<()> {
        let texc = self.canvas.texture_creator();
        let fmt = sdl2::pixels::PixelFormatEnum::RGB24;
        let mut tex =
            texc.create_texture_static(Some(fmt), frame.width as u32, frame.height as u32)?;
        let rect = sdl2::rect::Rect::new(0, 0, frame.width as u32, frame.height as u32);
        tex.update(
            Some(rect),
            Pixel::into_raw_slice(&frame.data),
            frame.width as usize * 3,
        )?;
        self.canvas.clear();
        self.canvas
            .copy(&tex, Some(rect), Some(rect))
            .map_err(|e| anyhow!(e))?;
        self.canvas.present();
        Ok(())
    }
}
