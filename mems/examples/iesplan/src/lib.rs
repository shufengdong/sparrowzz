use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::startpage::Dashboard;

pub mod startpage;

#[wasm_bindgen]
pub fn create_view(e: Element) {
    yew::Renderer::<Dashboard>::with_root(e).render();
}
