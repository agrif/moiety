use crate::shims::*;
use moiety::{
    display::{
        Bitmap,
        Display,
    },
    future::*,
    game::{
        Event,
        EventPump,
        Game,
    },
};
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

macro_rules! event_handler {
    ($pump:expr, $el:expr, {$($register:ident($evfn:expr),)*}) => {
        $({
            let mut send = $pump.sender();
            let onevent =
                Closure::wrap(Box::new(move |ev| {
                    futures::executor::block_on(
                        send.send($evfn(ev)),
                    );
                })
                              as Box<dyn FnMut(_)>);
            $el.$register(Some(onevent.as_ref().unchecked_ref()));
            onevent.forget();
        })*
    };
}

pub async fn run_web<G>(canvas: web_sys::HtmlCanvasElement, game: G)
where
    G: Game<'static, CanvasDisplay>,
    G::Error: std::fmt::Debug,
{
    let display = CanvasDisplay::new(canvas);

    let pump = EventPump::new();

    event_handler!(pump, display.canvas, {
        set_onmousedown(|ev: web_sys::MouseEvent| {
            Event::MouseDown(ev.offset_x(), ev.offset_y())
        }),
        set_onmouseup(|ev: web_sys::MouseEvent| {
            Event::MouseUp(ev.offset_x(), ev.offset_y())
        }),
    });

    let mut send = pump.sender();
    schedule(game.start(pump, display));
    await!(send.send(Event::Idle));
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
