mod a_component;
mod app;
mod js;
mod list;
mod list_item;
mod list_item_1;
mod list_item_2;
mod list_item_3;
mod reducer;
mod state;
use js::log;
mod rustact;
use crate::state::State;
use app::app;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
#[macro_use]
extern crate lazy_static;

lazy_static! {
#[derive(Debug, Default, Clone, Copy)]
static ref STORE: Mutex<rustact::RustactStore<State>> = Mutex::new(rustact::RustactStore::new(State::new(true)));
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let root_node = document
        .get_element_by_id("root")
        .expect("should have a root div")
        .append_child(&document.create_element("div").unwrap())
        .expect("couldn't append child");

    let app = app();
    rustact::render(app, &root_node);

    Ok(())
}
