pub mod animation;
pub mod character;
pub mod game_object;
pub mod player;
pub mod states;

use self::states::preloader::Preloader;
use micro_games_kit::{config::Config, game::GameInstance, GameLauncher};

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let config = Config::load_from_file("./assets/GameConfig.toml");
    #[cfg(target_arch = "wasm32")]
    let config = Config::load_from_str(include_str!("../assets/GameConfig.toml"));

    GameLauncher::new(GameInstance::new(Preloader))
        .title("top-down")
        .config(config.expect("Could not load Game Config!"))
        .run();
}
