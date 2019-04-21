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
#[macro_use]
mod shims;
use filesystem::*;
use shims::*;
mod format;
use format::*;
mod display;
use display::*;

#[wasm_bindgen]
pub fn go() -> js_sys::Promise { repromise(go_async()) }

pub async fn go_async() -> Result<JsValue, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");
    let body = document.body().expect("no body on document");

    let canvas: web_sys::HtmlCanvasElement =
        document.create_element("canvas")?.dyn_into().unwrap();
    body.append_child(&canvas)?;

    canvas.set_width(608);
    canvas.set_height(392);
    canvas.style().set_property("border", "1px solid black")?;

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

    let game = moiety::riven::Riven::new(rs);
    // FIXME error handling
    let mut runner = WebRunner::new(canvas);
    await!(runner.run(game)).unwrap();

    Ok(JsValue::NULL)
}
