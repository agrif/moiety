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

impl moiety::filesystem::AsyncRead for WebHandle {
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, IoResult<usize>> {
        fut!({
            let window = web_sys::window().unwrap();
            let request = web_sys::Request::new_with_str(&self.path).unwrap();
            request
                .headers()
                .set(
                    "Range",
                    &format!("bytes={}-{}", pos, pos as usize + buf.len() - 1),
                )
                .unwrap();

            let response: web_sys::Response =
                await!(unpromise(window.fetch_with_request(&request)))?;
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
    app.set_width(608);
    app.set_height(392);
    app.style().set_property("border", "1px solid black");
    body.append_child(&app)?;

    let ctx: web_sys::CanvasRenderingContext2d =
        app.get_context("2d")?.unwrap().dyn_into().unwrap();

    let fs = WebFilesystem::new("local/mhk");
    let map = moiety::MhkMap::new(fs, moiety::riven::map_5cd());
    let fmt = moiety::MhkFormat;
    // let fmt = moiety::riven::Format {
    //     blst: moiety::JsonFormat,
    //     card: moiety::JsonFormat,
    //     name: moiety::JsonFormat,
    //     plst: moiety::JsonFormat,
    //     tbmp: moiety::PngFormat,
    // };

    let rs = moiety::Resources::new(map, fmt);

    let r = await!(rs.open(
        moiety::riven::Stack::B,
        moiety::riven::Resource::TBMP,
        50042
    ))
    .unwrap();

    let withalpha: Vec<palette::Alpha<palette::Srgb<u8>, u8>> = r
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

    log!("size: {} x {}", imdata.width(), imdata.height());
    ctx.put_image_data(&imdata, 0.0, 0.0)?;

    // app.set_inner_html(&format!("{:#?}", r.width));

    Ok(JsValue::NULL)
}
