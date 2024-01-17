use crate::context::GameContext;
#[cfg(not(target_arch = "wasm32"))]
use glutin::{event::Event, window::Window};
#[cfg(target_arch = "wasm32")]
use instant::Instant;
use spitfire_draw::{
    context::DrawContext,
    utils::{ShaderRef, Vertex},
};
use spitfire_glow::{app::AppState, graphics::Graphics, renderer::GlowBlending};
use spitfire_gui::context::GuiContext;
use spitfire_input::InputContext;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use winit::{event::Event, window::Window};

#[derive(Default)]
pub enum GameStateChange {
    #[default]
    Continue,
    Swap(Box<dyn GameState>),
    Push(Box<dyn GameState>),
    Pop,
}

#[allow(unused_variables)]
pub trait GameState {
    fn enter(&mut self, context: GameContext) {}

    fn exit(&mut self, context: GameContext) {}

    fn update(&mut self, context: GameContext, delta_time: f32) {}

    fn fixed_update(&mut self, context: GameContext, delta_time: f32) {}

    fn draw(&mut self, context: GameContext) {}

    fn draw_gui(&mut self, context: GameContext) {}
}

pub struct GameInstance {
    pub fixed_delta_time: f32,
    pub color_shader: &'static str,
    pub image_shader: &'static str,
    pub text_shader: &'static str,
    draw: DrawContext,
    gui: GuiContext,
    input: InputContext,
    timer: Instant,
    fixed_timer: Instant,
    states: Vec<Box<dyn GameState>>,
    state_change: GameStateChange,
}

impl Default for GameInstance {
    fn default() -> Self {
        Self {
            fixed_delta_time: 1.0 / 60.0,
            color_shader: "color",
            image_shader: "image",
            text_shader: "text",
            draw: Default::default(),
            gui: Default::default(),
            input: Default::default(),
            timer: Instant::now(),
            fixed_timer: Instant::now(),
            states: Default::default(),
            state_change: Default::default(),
        }
    }
}

impl GameInstance {
    pub fn new(state: impl GameState + 'static) -> Self {
        Self {
            state_change: GameStateChange::Push(Box::new(state)),
            ..Default::default()
        }
    }

    pub fn with_fixed_time_step(mut self, value: f32) -> Self {
        self.fixed_delta_time = value;
        self
    }

    pub fn with_fps(mut self, frames_per_second: usize) -> Self {
        self.set_fps(frames_per_second);
        self
    }

    pub fn with_color_shader(mut self, name: &'static str) -> Self {
        self.color_shader = name;
        self
    }

    pub fn with_image_shader(mut self, name: &'static str) -> Self {
        self.image_shader = name;
        self
    }

    pub fn with_text_shader(mut self, name: &'static str) -> Self {
        self.text_shader = name;
        self
    }

    pub fn fps(&self) -> usize {
        (1.0 / self.fixed_delta_time).ceil() as usize
    }

    pub fn set_fps(&mut self, frames_per_second: usize) {
        self.fixed_delta_time = 1.0 / frames_per_second as f32;
    }

    pub fn process_frame(&mut self, graphics: &mut Graphics<Vertex>) {
        let delta_time = self.timer.elapsed().as_secs_f32();
        if let Some(state) = self.states.last_mut() {
            state.update(
                GameContext {
                    graphics,
                    draw: &mut self.draw,
                    gui: &mut self.gui,
                    input: &mut self.input,
                    state_change: &mut self.state_change,
                },
                delta_time,
            );
        }

        let fixed_delta_time = self.fixed_timer.elapsed().as_secs_f32();
        if fixed_delta_time > self.fixed_delta_time {
            self.timer = Instant::now();
            if let Some(state) = self.states.last_mut() {
                state.fixed_update(
                    GameContext {
                        graphics,
                        draw: &mut self.draw,
                        gui: &mut self.gui,
                        input: &mut self.input,
                        state_change: &mut self.state_change,
                    },
                    fixed_delta_time,
                );
            }
        }

        self.draw.begin_frame(graphics);
        self.draw.push_shader(&ShaderRef::name(self.image_shader));
        self.draw.push_blending(GlowBlending::Alpha);
        if let Some(state) = self.states.last_mut() {
            state.draw(GameContext {
                graphics,
                draw: &mut self.draw,
                gui: &mut self.gui,
                input: &mut self.input,
                state_change: &mut self.state_change,
            });
        }
        self.gui.begin_frame();
        if let Some(state) = self.states.last_mut() {
            state.draw_gui(GameContext {
                graphics,
                draw: &mut self.draw,
                gui: &mut self.gui,
                input: &mut self.input,
                state_change: &mut self.state_change,
            });
        }
        self.gui.end_frame(
            &mut self.draw,
            graphics,
            &ShaderRef::name(self.color_shader),
            &ShaderRef::name(self.image_shader),
            &ShaderRef::name(self.text_shader),
        );
        self.draw.end_frame();
        self.input.maintain();

        match std::mem::take(&mut self.state_change) {
            GameStateChange::Continue => {}
            GameStateChange::Swap(mut state) => {
                if let Some(mut state) = self.states.pop() {
                    state.exit(GameContext {
                        graphics,
                        draw: &mut self.draw,
                        gui: &mut self.gui,
                        input: &mut self.input,
                        state_change: &mut self.state_change,
                    });
                }
                state.enter(GameContext {
                    graphics,
                    draw: &mut self.draw,
                    gui: &mut self.gui,
                    input: &mut self.input,
                    state_change: &mut self.state_change,
                });
                self.states.push(state);
                self.timer = Instant::now();
            }
            GameStateChange::Push(mut state) => {
                state.enter(GameContext {
                    graphics,
                    draw: &mut self.draw,
                    gui: &mut self.gui,
                    input: &mut self.input,
                    state_change: &mut self.state_change,
                });
                self.states.push(state);
                self.timer = Instant::now();
            }
            GameStateChange::Pop => {
                if let Some(mut state) = self.states.pop() {
                    state.exit(GameContext {
                        graphics,
                        draw: &mut self.draw,
                        gui: &mut self.gui,
                        input: &mut self.input,
                        state_change: &mut self.state_change,
                    });
                }
                self.timer = Instant::now();
            }
        }
    }

    pub fn process_event(&mut self, event: &Event<()>) -> bool {
        if let Event::WindowEvent { event, .. } = event {
            self.input.on_event(event);
        }
        !self.states.is_empty() || !matches!(self.state_change, GameStateChange::Continue)
    }
}

impl AppState<Vertex> for GameInstance {
    fn on_redraw(&mut self, graphics: &mut Graphics<Vertex>) {
        self.process_frame(graphics);
    }

    fn on_event(&mut self, event: Event<()>, _: &mut Window) -> bool {
        self.process_event(&event)
    }
}
