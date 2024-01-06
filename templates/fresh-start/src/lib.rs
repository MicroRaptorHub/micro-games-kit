#![cfg(target_arch = "wasm32")]

mod game;
pub mod states;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    game::main();
}
