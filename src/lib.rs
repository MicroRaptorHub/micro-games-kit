pub mod third_party {
    pub use fontdue;
    pub use glutin;
    pub use image;
    pub use raui_core;
    pub use raui_immediate;
    pub use raui_immediate_widgets;
    pub use serde;
    pub use spitfire_core;
    pub use spitfire_draw;
    pub use spitfire_fontdue;
    pub use spitfire_glow;
    pub use spitfire_gui;
    pub use spitfire_input;
    pub use toml;
    pub use vek;
}

pub mod config;
pub mod context;
pub mod game;
pub mod loader;

use config::Config;
use game::GameInstance;
use spitfire_draw::utils::Vertex;
use spitfire_glow::app::App;
use std::{error::Error, path::Path};

pub struct GameLauncher {
    instance: GameInstance,
    title: String,
    config: Config,
}

impl GameLauncher {
    pub fn new(instance: GameInstance) -> Self {
        Self {
            instance,
            title: "MicroGamesKit".to_owned(),
            config: Config::default(),
        }
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn load_config(mut self, config: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        self.config = Config::load(config)?;
        Ok(self)
    }

    pub fn run(self) {
        #[cfg(debug_assertions)]
        println!("* Game {:#?}", self.config);
        App::<Vertex>::new(self.config.to_app_config(self.title)).run(self.instance);
    }
}
