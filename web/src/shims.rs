use futures::{
    compat::Future01CompatExt,
    FutureExt,
    TryFutureExt,
};
use js_sys::Promise;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};

// pretend console.log is println!
#[macro_export]
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
pub async fn unpromise<T>(p: Promise) -> Result<T, std::io::Error>
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

// promise wrapping
pub fn repromise<F, R, E>(fut: F) -> Promise
where
    F: futures::Future<Output = Result<R, E>> + 'static,
    R: JsCast,
    E: JsCast,
{
    let cast = fut.map(|re| {
        re.map_err(|e| {
            e.dyn_into()
                .unwrap_or(JsValue::from_str("could not convect error"))
        })
        .and_then(|r| {
            r.dyn_into()
                .map_err(|_| JsValue::from_str("could not convert result"))
        })
    });
    wasm_bindgen_futures::future_to_promise(cast.boxed().compat())
}

pub fn schedule<F, R, E>(fut: F) -> Promise
where
    F: futures::Future<Output = Result<R, E>> + 'static,
    E: std::fmt::Debug,
{
    repromise(
        fut.map_ok(|_| JsValue::NULL)
            .map_err(|e| JsValue::from_str(&format!("error: {:?}", e))),
    )
}
