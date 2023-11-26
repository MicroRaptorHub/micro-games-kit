use serde::Deserialize;
use spitfire_glow::app::AppConfig;
use std::{error::Error, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_width")]
    pub width: u32,
    #[serde(default = "Config::default_height")]
    pub height: u32,
    #[serde(default = "Config::default_fullscreen")]
    pub fullscreen: bool,
    #[serde(default = "Config::default_maximized")]
    pub maximized: bool,
    #[serde(default = "Config::default_vsync")]
    pub vsync: bool,
    #[serde(default = "Config::default_double_buffer")]
    pub double_buffer: Option<bool>,
    #[serde(default = "Config::default_hardware_acceleration")]
    pub hardware_acceleration: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: Self::default_width(),
            height: Self::default_height(),
            fullscreen: Self::default_fullscreen(),
            maximized: Self::default_maximized(),
            vsync: Self::default_vsync(),
            double_buffer: Self::default_double_buffer(),
            hardware_acceleration: Self::default_hardware_acceleration(),
        }
    }
}

impl Config {
    fn default_width() -> u32 {
        1024
    }

    fn default_height() -> u32 {
        576
    }

    fn default_fullscreen() -> bool {
        true
    }

    fn default_maximized() -> bool {
        false
    }

    fn default_vsync() -> bool {
        true
    }

    fn default_double_buffer() -> Option<bool> {
        None
    }

    fn default_hardware_acceleration() -> Option<bool> {
        Some(true)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn to_app_config(&self, name: impl ToString) -> AppConfig {
        AppConfig {
            title: name.to_string(),
            width: self.width,
            height: self.height,
            fullscreen: self.fullscreen,
            maximized: self.maximized,
            vsync: self.vsync,
            double_buffer: self.double_buffer,
            hardware_acceleration: self.hardware_acceleration,
            color: [0.0, 0.0, 0.0],
            ..Default::default()
        }
    }
}
