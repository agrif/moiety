use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn go() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");
    let body = document.body().expect("no body on document");

    let app = document.create_element("ul")?;
    body.append_child(&app)?;

    for s in <moiety::riven::Stack as moiety::Stack>::all() {
        let part = document.create_element("li")?;
        part.set_inner_html(&format!("{:?}", s));
        app.append_child(&part)?;
    }

    Ok(())
}
