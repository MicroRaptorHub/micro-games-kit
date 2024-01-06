#![cfg(not(target_arch = "wasm32"))]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;
pub mod states;

fn main() {
    game::main();
}
