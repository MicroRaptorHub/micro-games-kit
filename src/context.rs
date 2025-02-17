use crate::{audio::Audio, game::GameStateChange};
use keket::database::AssetDatabase;
use spitfire_draw::{context::DrawContext, utils::Vertex};
use spitfire_glow::graphics::Graphics;
use spitfire_gui::context::GuiContext;
use spitfire_input::InputContext;

pub struct GameContext<'a> {
    pub graphics: &'a mut Graphics<Vertex>,
    pub draw: &'a mut DrawContext,
    pub gui: &'a mut GuiContext,
    pub input: &'a mut InputContext,
    pub state_change: &'a mut GameStateChange,
    pub assets: &'a mut AssetDatabase,
    pub audio: &'a mut Audio,
}

impl<'a> GameContext<'a> {
    pub fn fork(&'a mut self) -> Self {
        Self {
            graphics: self.graphics,
            draw: self.draw,
            gui: self.gui,
            input: self.input,
            state_change: self.state_change,
            assets: self.assets,
            audio: self.audio,
        }
    }
}
