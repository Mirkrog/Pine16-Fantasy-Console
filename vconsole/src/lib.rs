#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod console;
#[cfg(target_arch = "wasm32")]
mod renderer;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::UnwrapThrowExt;

    console_error_panic_hook::set_once();
    app::run(false).unwrap_throw();

    Ok(())
}
