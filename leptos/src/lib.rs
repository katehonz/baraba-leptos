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
pub mod bg_amount_words;

pub use app::*;
pub use router::*;

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
