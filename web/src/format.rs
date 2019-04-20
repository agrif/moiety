use crate::{
    shims::*,
    WebHandle,
};
use moiety::future::*;
use std::io::Result;
use wasm_bindgen::JsCast;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct WebFormat;

// to get pixel data out of a url, we need to draw it to a canvas
// (yes, really)
// so make a canvas here we can re-use
thread_local! {
    static WEB_FORMAT_CANVAS: web_sys::HtmlCanvasElement = {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas: web_sys::HtmlCanvasElement = document.create_element("canvas").unwrap().dyn_into().unwrap();
        canvas.set_width(1024);
        canvas.set_height(1024);
        canvas
    };
}

impl moiety::Format<WebHandle> for WebFormat {
    type Error = std::io::Error;
}

impl
    moiety::FormatFor<
        WebHandle,
        moiety::riven::Resource<moiety::display::Bitmap>,
    > for WebFormat
{
    fn convert<'a>(
        &'a self,
        input: WebHandle,
    ) -> Fut<'a, Result<moiety::display::Bitmap>>
    where
        WebHandle: 'a,
    {
        fut!({
            let window = web_sys::window().unwrap();
            let response = await!(input.make_request(0, None))?;
            let blob: web_sys::Blob =
                await!(unpromise(unerror(response.blob())?))?;
            let im: web_sys::ImageBitmap = await!(unpromise(unerror(
                window.create_image_bitmap_with_blob(&blob)
            )?))?;

            let imdata = WEB_FORMAT_CANVAS.with(|canvas| -> Result<_> {
                let ctx: web_sys::CanvasRenderingContext2d =
                    unerror(canvas.get_context("2d"))?
                        .unwrap()
                        .dyn_into()
                        .unwrap();
                unerror(ctx.draw_image_with_image_bitmap(&im, 0.0, 0.0))?;
                Ok(unerror(ctx.get_image_data(
                    0.0,
                    0.0,
                    im.width() as f64,
                    im.height() as f64,
                ))?
                .data())
            })?;

            let nicer: &[palette::Srgba<u8>] =
                palette::Pixel::from_raw_slice(&imdata);
            Ok(moiety::display::Bitmap {
                width: im.width() as u16,
                height: im.height() as u16,
                palette: None,
                data: nicer.iter().map(|p| p.color).collect(),
            })
        })
    }

    fn extension<'a>(&'a self) -> Option<&'a str> { Some(&".png") }
}
