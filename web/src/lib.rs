#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]

#[macro_use]
extern crate moiety;

use futures::{
    compat::Future01CompatExt,
    future::{
        FutureExt,
        TryFutureExt,
    },
};
use moiety::{
    filesystem::{
        AsyncRead,
        Filesystem,
    },
    future::*,
};
use std::io::Result as IoResult;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

// pretend console.log is println!
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// error wrangling
pub fn unerror<T>(r: Result<T, JsValue>) -> Result<T, std::io::Error> {
    r.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e))
    })
}

// future wrangling
pub async fn unpromise<T>(p: js_sys::Promise) -> Result<T, std::io::Error>
where
    T: JsCast + Sized,
{
    unerror(await!(wasm_bindgen_futures::JsFuture::from(p).compat())).and_then(
        |v| {
            v.dyn_into().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("unexpected type: {:?}", e),
                )
            })
        },
    )
}

#[derive(Debug)]
pub struct WebFilesystem {
    pub root: String,
}

impl WebFilesystem {
    pub fn new<S>(root: S) -> Self
    where
        S: AsRef<str>,
    {
        WebFilesystem {
            root: root.as_ref().to_owned(),
        }
    }
}

impl moiety::filesystem::Filesystem for WebFilesystem {
    type Handle = WebHandle;

    fn open<'a>(&'a self, path: &'a [&str]) -> Fut<'a, IoResult<Self::Handle>> {
        fut!({
            Ok(WebHandle {
                path: format!("{}/{}", self.root, path.join("/")),
            })
        })
    }
}

pub struct WebHandle {
    pub path: String,
}

impl WebHandle {
    pub async fn make_request(
        &self,
        start: u64,
        end: Option<u64>,
    ) -> IoResult<web_sys::Response> {
        let window = web_sys::window().unwrap();
        let request = web_sys::Request::new_with_str(&self.path).unwrap();
        if let Some(endi) = end {
            request
                .headers()
                .set("Range", &format!("bytes={}-{}", start, endi - 1))
                .unwrap();
        } else if start > 0 {
            request
                .headers()
                .set("Range", &format!("bytes={}-", start))
                .unwrap();
        }

        await!(unpromise(window.fetch_with_request(&request)))
    }
}

impl moiety::filesystem::AsyncRead for WebHandle {
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, IoResult<usize>> {
        fut!({
            let response =
                await!(self.make_request(pos, Some(pos + buf.len() as u64)))?;
            if response.status() == 416 {
                // we requested something of 0 bytes...
                return Ok(0);
            }
            let arrbuf: js_sys::ArrayBuffer =
                await!(unpromise(unerror(response.array_buffer())?))?;
            let arr = js_sys::Uint8Array::new(&arrbuf);
            let read = arr.length() as usize;
            assert!(buf.len() >= read);
            arr.copy_to(&mut buf[..read]);
            Ok(read)
        })
    }

    fn read_until_end_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut Vec<u8>,
    ) -> Fut<'a, IoResult<usize>> {
        fut!({
            let response = await!(self.make_request(pos, None))?;
            let arrbuf: js_sys::ArrayBuffer =
                await!(unpromise(unerror(response.array_buffer())?))?;
            let arr = js_sys::Uint8Array::new(&arrbuf);
            let read = arr.length() as usize;
            buf.reserve(read);
            let start = buf.len();
            unsafe {
                buf.set_len(start + read);
                arr.copy_to(&mut buf[start..]);
            }
            Ok(read)
        })
    }
}

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
    ) -> Fut<'a, IoResult<moiety::display::Bitmap>>
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

            let imdata = WEB_FORMAT_CANVAS.with(|canvas| -> IoResult<_> {
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

#[wasm_bindgen]
pub fn go() -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(go_async().boxed().compat())
}

pub async fn go_async() -> Result<JsValue, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");
    let body = document.body().expect("no body on document");

    let app: web_sys::HtmlCanvasElement =
        document.create_element("canvas")?.dyn_into().unwrap();
    body.append_child(&app)?;
    let div: web_sys::HtmlElement =
        document.create_element("pre")?.dyn_into().unwrap();
    body.append_child(&div)?;

    app.set_width(608);
    app.set_height(392);
    app.style().set_property("border", "1px solid black")?;

    let ctx: web_sys::CanvasRenderingContext2d =
        app.get_context("2d")?.unwrap().dyn_into().unwrap();

    // let fs = WebFilesystem::new("local/mhk");
    // let map = moiety::MhkMap::new(fs, moiety::riven::map_5cd());
    // let fmt = moiety::MhkFormat;

    let fs = WebFilesystem::new("local");
    let map = moiety::DirectMap::new(fs);
    let fmt = moiety::riven::Format {
        blst: moiety::JsonFormat,
        card: moiety::JsonFormat,
        name: moiety::JsonFormat,
        plst: moiety::JsonFormat,
        tbmp: WebFormat,
    };

    let rs = moiety::Resources::new(map, fmt);

    let r = await!(rs.open(
        moiety::riven::Stack::B,
        moiety::riven::Resource::TBMP,
        50042
    ))
    .unwrap();

    let withalpha: Vec<palette::Srgba<u8>> = r
        .data
        .iter()
        .map(|p| {
            palette::Alpha {
                color: *p,
                alpha: 255,
            }
        })
        .collect();
    let mut rawbuf: Vec<u8> = palette::Pixel::into_raw_slice(&withalpha)
        .iter()
        .cloned()
        .collect();
    let imdata = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&mut rawbuf[..]),
        r.width as u32,
        r.height as u32,
    )?;

    ctx.put_image_data(&imdata, 0.0, 0.0)?;

    let card = await!(rs.open(
        moiety::riven::Stack::J,
        moiety::riven::Resource::CARD,
        43,
    ))
    .unwrap();
    div.set_inner_html(&format!("{:#?}", card));

    Ok(JsValue::NULL)
}
