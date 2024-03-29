pub mod states;

use self::states::gameplay::Gameplay;
use micro_games_kit::{config::Config, game::GameInstance, GameLauncher};

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let config = Config::load_from_file("./assets/GameConfig.toml");
    #[cfg(target_arch = "wasm32")]
    let config = Config::load_from_str(include_str!("../assets/GameConfig.toml"));

    GameLauncher::new(GameInstance::new(Gameplay::default()))
        .title("Micro Game")
        .config(config.expect("Could not load Game Config!"))
        .run();
}
