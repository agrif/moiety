#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]

#[macro_use]
extern crate moiety;

use futures::{
    FutureExt,
    TryFutureExt,
};
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

mod filesystem;
mod shims;
use filesystem::*;
mod format;
use format::*;

// pretend console.log is println!
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
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
