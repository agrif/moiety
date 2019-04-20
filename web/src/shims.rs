use futures::compat::Future01CompatExt;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

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
