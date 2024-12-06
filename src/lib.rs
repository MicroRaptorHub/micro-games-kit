pub mod third_party {
    pub use anim8;
    pub use emergent;
    pub use fontdue;
    pub use gilrs;
    #[cfg(not(target_arch = "wasm32"))]
    pub use glutin as windowing;
    pub use image;
    #[cfg(target_arch = "wasm32")]
    pub use instant::Instant;
    pub use intuicio_backend_vm;
    pub use intuicio_core;
    pub use intuicio_data;
    pub use intuicio_derive;
    pub use intuicio_framework_ecs;
    pub use intuicio_frontend_simpleton;
    pub use kira;
    pub use noise;
    pub use rand;
    pub use raui_core;
    pub use raui_immediate;
    pub use raui_immediate_widgets;
    pub use rstar;
    pub use serde;
    pub use spitfire_core;
    pub use spitfire_draw;
    pub use spitfire_fontdue;
    pub use spitfire_glow;
    pub use spitfire_gui;
    pub use spitfire_input;
    #[cfg(not(target_arch = "wasm32"))]
    pub use std::time::Instant;
    pub use toml;
    pub use typid;
    pub use vek;
    #[cfg(target_arch = "wasm32")]
    pub use winit as windowing;
}

pub mod animation;
pub mod character;
pub mod config;
pub mod context;
pub mod game;
pub mod gamepad;
pub mod grid_world;
pub mod loader;
pub mod pcg;
pub mod scripting;
pub mod tag;

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

    pub fn load_config_from_file(
        mut self,
        config: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn Error>> {
        self.config = Config::load_from_file(config)?;
        Ok(self)
    }

    pub fn load_config_from_str(mut self, config: &str) -> Result<Self, Box<dyn Error>> {
        self.config = Config::load_from_str(config)?;
        Ok(self)
    }

    pub fn run(self) {
        #[cfg(debug_assertions)]
        spitfire_glow::console_log!("* Game {:#?}", self.config);
        App::<Vertex>::new(self.config.to_app_config(self.title)).run(self.instance);
    }
}
