use crate::shims::*;
use futures::{
    sink::SinkExt,
    stream::StreamExt,
};
use moiety::{
    display::{
        Bitmap,
        Display,
    },
    future::*,
    game::{
        Event,
        Game,
    },
};
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

pub struct WebRunner {
    display: CanvasDisplay,
}

impl WebRunner {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        WebRunner {
            display: CanvasDisplay::new(canvas),
        }
    }

    pub async fn run<G>(&mut self, mut game: G) -> Result<(), G::Error>
    where
        G: Game<CanvasDisplay>,
    {
        let (mut send, mut recv) = futures::channel::mpsc::channel(128);

        if !await!(game.handle(&Event::Idle, &mut self.display))? {
            return Ok(());
        }

        let mut mousedown_send = send.clone();
        let onmousedown =
            Closure::wrap(Box::new(move |ev: web_sys::MouseEvent| {
                futures::executor::block_on(
                    mousedown_send
                        .send(Event::MouseDown(ev.offset_x(), ev.offset_y())),
                )
                .unwrap();
            })
                as Box<dyn FnMut(web_sys::MouseEvent)>);
        self.display
            .canvas
            .set_onmousedown(Some(onmousedown.as_ref().unchecked_ref()));
        onmousedown.forget();

        let mut mouseup_send = send.clone();
        let onmouseup =
            Closure::wrap(Box::new(move |ev: web_sys::MouseEvent| {
                futures::executor::block_on(
                    mouseup_send
                        .send(Event::MouseUp(ev.offset_x(), ev.offset_y())),
                )
                .unwrap();
            })
                as Box<dyn FnMut(web_sys::MouseEvent)>);
        self.display
            .canvas
            .set_onmouseup(Some(onmouseup.as_ref().unchecked_ref()));
        onmouseup.forget();

        await!(send.send(Event::Idle)).unwrap();
        while let Some(ev) = await!(recv.next()) {
            let running = await!(game.handle(&ev, &mut self.display))?;
            if !running || ev == Event::Quit {
                break;
            }
        }

        Ok(())
    }
}

pub struct CanvasDisplay {
    canvas: web_sys::HtmlCanvasElement,
    ctx: web_sys::CanvasRenderingContext2d,
}

impl CanvasDisplay {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        CanvasDisplay {
            ctx: canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap(),
            canvas,
        }
    }
}

impl Display for CanvasDisplay {
    type Bitmap = web_sys::ImageData;
    type Error = std::io::Error;

    fn transfer<'a>(
        &'a self,
        src: &'a Bitmap,
    ) -> Fut<'a, Result<Self::Bitmap, Self::Error>> {
        fut!({
            let withalpha: Vec<palette::Srgba<u8>> = src
                .data
                .iter()
                .map(|p| {
                    palette::Alpha {
                        color: *p,
                        alpha: 255,
                    }
                })
                .collect();
            let mut rawbuf: Vec<u8> =
                palette::Pixel::into_raw_slice(&withalpha)
                    .iter()
                    .cloned()
                    .collect();
            let imdata =
                unerror(web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                    wasm_bindgen::Clamped(&mut rawbuf[..]),
                    src.width as u32,
                    src.height as u32,
                ))?;
            Ok(imdata)
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
        let width = (right - left) as f64;
        let height = (bottom - top) as f64;
        self.ctx.put_image_data_with_dirty_x_and_dirty_y_and_dirty_width_and_dirty_height(src, left as f64, top as f64, 0.0, 0.0, width, height).unwrap();
    }

    fn flip(&mut self) {
        // hmmmmmm
    }
}
