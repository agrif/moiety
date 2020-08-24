use crate::Bitmap;

use anyhow::Result;

pub enum Event {
    Idle,
    Exit,
    MouseDown(i32, i32),
}

#[async_trait::async_trait(?Send)]
pub trait Game {
    fn window_title(&self) -> &str {
        "Moiety"
    }

    fn window_size(&self) -> (u32, u32);

    async fn start(&mut self, ctx: &mut Context) -> Result<()>;
    async fn handle_event(&mut self, ctx: &mut Context, ev: Event) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait GameRunner {
    async fn output(&mut self, frame: &Bitmap) -> Result<()>;
}

pub struct Context {
    runner: Box<dyn GameRunner>,
    framebuffer: Bitmap,
}

impl Context {
    pub fn new<G, R>(game: &G, runner: R) -> Self
    where
        G: Game,
        R: GameRunner + 'static,
    {
        let window_size = game.window_size();
        let fbsize = window_size.0 as usize * window_size.1 as usize;
        Context {
            runner: Box::new(runner),
            framebuffer: Bitmap {
                width: window_size.0 as u16,
                height: window_size.1 as u16,
                palette: None,
                data: vec![palette::Srgb::new(0, 0, 0); fbsize],
            }
        }
    }

    pub fn draw(&mut self, bmp: &Bitmap, left: u16, top: u16, mut right: u16, mut bottom: u16) {
        if right <= left || bottom <= top {
            return;
        }
        if right > self.framebuffer.width {
            right = self.framebuffer.width;
        }
        if bottom > self.framebuffer.height {
            bottom = self.framebuffer.height;
        }
        if right > left + bmp.width {
            right = left + bmp.width;
        }
        if bottom > top + bmp.height {
            bottom = top + bmp.height;
        }
        let width = right - left;
        let height = bottom - top;
        let srcstride = (bmp.width - width) as usize;
        let dststride = (self.framebuffer.width - width) as usize;
        let mut srci = 0;
        let mut dsti = (top * bmp.width + left) as usize;
        for _ in 0..height {
            for _ in 0..width {
                self.framebuffer.data[dsti] = bmp.data[srci];
                srci += 1;
                dsti += 1;
            }
            srci += srcstride;
            dsti += dststride;
        }
    }

    pub async fn transition(&mut self) -> Result<()> {
        self.runner.output(&self.framebuffer).await
    }
}
