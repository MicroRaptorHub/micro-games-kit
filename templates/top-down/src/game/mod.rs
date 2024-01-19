pub mod enemy;
pub mod item;
pub mod player;
pub mod states;
pub mod torch;
pub mod ui;
pub mod utils;

use self::states::preloader::Preloader;
use micro_games_kit::{config::Config, game::GameInstance, GameLauncher};

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let config = Config::load_from_file("./assets/GameConfig.toml");
    #[cfg(target_arch = "wasm32")]
    let config = Config::load_from_str(include_str!("../../assets/GameConfig.toml"));

    GameLauncher::new(GameInstance::new(Preloader))
        .title("RED HOOD")
        .config(config.expect("Could not load Game Config!"))
        .run();
}
