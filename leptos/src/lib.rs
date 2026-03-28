use wasm_bindgen::prelude::*;

pub mod app;
pub mod models;
pub mod api;
pub mod stores;
pub mod pages;
pub mod components;
pub mod router;
pub mod i18n;
pub mod constants;
pub mod context;

pub use app::*;
pub use router::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
