use micro_games_kit::{
    config::Config,
    context::GameContext,
    game::{GameInstance, GameState, GameStateChange},
    load_asset,
    loader::{load_font, load_shader, load_texture},
    third_party::{
        raui_core::layout::CoordsMappingScaling,
        raui_immediate_widgets::core::{
            text_box, Color, TextBoxFont, TextBoxHorizontalAlign, TextBoxProps,
            TextBoxVerticalAlign,
        },
        spitfire_draw::{
            sprite::{Sprite, SpriteTexture},
            utils::{Drawable, TextureRef},
        },
        spitfire_glow::{
            graphics::{CameraScaling, Shader},
            renderer::GlowTextureFiltering,
        },
        spitfire_input::{
            CardinalInputCombinator, InputActionRef, InputConsume, InputMapping, VirtualAction,
        },
        vek::Vec2,
        windowing::event::VirtualKeyCode,
    },
    GameLauncher,
};
use std::error::Error;

const SPEED: f32 = 100.0;

#[derive(Default)]
struct Preloader;

impl GameState for Preloader {
    fn enter(&mut self, context: GameContext) {
        context.graphics.color = [0.2, 0.2, 0.2];
        context.graphics.main_camera.screen_alignment = 0.5.into();
        context.graphics.main_camera.scaling = CameraScaling::FitVertical(500.0);
        context.gui.coords_map_scaling = CoordsMappingScaling::FitVertical(500.0);

        load_shader(
            context.draw,
            context.graphics,
            "color",
            Shader::COLORED_VERTEX_2D,
            Shader::PASS_FRAGMENT,
        );
        load_shader(
            context.draw,
            context.graphics,
            "image",
            Shader::TEXTURED_VERTEX_2D,
            Shader::TEXTURED_FRAGMENT,
        );
        load_shader(
            context.draw,
            context.graphics,
            "text",
            Shader::TEXT_VERTEX,
            Shader::TEXT_FRAGMENT,
        );

        load_texture(
            context.draw,
            context.graphics,
            "ferris",
            load_asset!("./resources/ferris.png"),
            1,
            1,
        );

        load_font(
            context.draw,
            "roboto",
            load_asset!("./resources/Roboto-Regular.ttf"),
        );

        *context.state_change = GameStateChange::Swap(Box::new(State::default()));
    }
}

#[derive(Default)]
struct State {
    ferris: Sprite,
    movement: CardinalInputCombinator,
    exit: InputActionRef,
}

impl GameState for State {
    fn enter(&mut self, context: GameContext) {
        self.ferris = Sprite::single(SpriteTexture {
            sampler: "u_image".into(),
            texture: TextureRef::name("ferris"),
            filtering: GlowTextureFiltering::Linear,
        })
        .pivot(0.5.into());

        let move_left = InputActionRef::default();
        let move_right = InputActionRef::default();
        let move_up = InputActionRef::default();
        let move_down = InputActionRef::default();
        self.movement = CardinalInputCombinator::new(
            move_left.clone(),
            move_right.clone(),
            move_up.clone(),
            move_down.clone(),
        );
        context.input.push_mapping(
            InputMapping::default()
                .consume(InputConsume::Hit)
                .action(
                    VirtualAction::KeyButton(VirtualKeyCode::A),
                    move_left.clone(),
                )
                .action(
                    VirtualAction::KeyButton(VirtualKeyCode::D),
                    move_right.clone(),
                )
                .action(VirtualAction::KeyButton(VirtualKeyCode::W), move_up.clone())
                .action(
                    VirtualAction::KeyButton(VirtualKeyCode::S),
                    move_down.clone(),
                )
                .action(VirtualAction::KeyButton(VirtualKeyCode::Left), move_left)
                .action(VirtualAction::KeyButton(VirtualKeyCode::Right), move_right)
                .action(VirtualAction::KeyButton(VirtualKeyCode::Up), move_up)
                .action(VirtualAction::KeyButton(VirtualKeyCode::Down), move_down)
                .action(
                    VirtualAction::KeyButton(VirtualKeyCode::Escape),
                    self.exit.clone(),
                ),
        );
    }

    fn exit(&mut self, context: GameContext) {
        context.input.pop_mapping();
    }

    fn update(&mut self, context: GameContext, delta_time: f32) {
        let movement = Vec2::<f32>::from(self.movement.get());
        self.ferris.transform.position += movement * SPEED * delta_time;

        if self.exit.get().is_pressed() {
            *context.state_change = GameStateChange::Pop;
        }
    }

    fn draw(&mut self, context: GameContext) {
        self.ferris.draw(context.draw, context.graphics);
    }

    fn draw_gui(&mut self, _: GameContext) {
        text_box(TextBoxProps {
            text: "Hello, World!".to_owned(),
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Bottom,
            font: TextBoxFont {
                name: "roboto".to_owned(),
                size: 50.0,
            },
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            ..Default::default()
        });
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    GameLauncher::new(GameInstance::new(Preloader::default()))
        .title("Hello World!")
        .config(Config::load_from_file("./resources/GameConfig.toml")?)
        .run();
    Ok(())
}
