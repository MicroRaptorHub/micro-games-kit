use micro_games_kit::{
    animation::spine::SpineSkeleton,
    assets::{make_directory_database, shader::ShaderAsset, spine::SpineAsset},
    config::Config,
    context::GameContext,
    game::{GameInstance, GameState, GameStateChange},
    third_party::{
        spitfire_draw::utils::Drawable,
        spitfire_glow::graphics::{CameraScaling, Shader},
    },
    GameLauncher,
};
use std::error::Error;

#[derive(Default)]
struct Preloader;

impl GameState for Preloader {
    fn enter(&mut self, context: GameContext) {
        context.graphics.color = [0.2, 0.2, 0.2, 1.0];
        context.graphics.main_camera.screen_alignment = 0.5.into();
        context.graphics.main_camera.scaling = CameraScaling::FitVertical(500.0);

        context
            .assets
            .spawn(
                "shader://color",
                (ShaderAsset::new(
                    Shader::COLORED_VERTEX_2D,
                    Shader::PASS_FRAGMENT,
                ),),
            )
            .unwrap();
        context
            .assets
            .spawn(
                "shader://image",
                (ShaderAsset::new(
                    Shader::TEXTURED_VERTEX_2D,
                    Shader::TEXTURED_FRAGMENT,
                ),),
            )
            .unwrap();
        context
            .assets
            .spawn(
                "shader://text",
                (ShaderAsset::new(Shader::TEXT_VERTEX, Shader::TEXT_FRAGMENT),),
            )
            .unwrap();

        context.assets.ensure("spine://robot.zip").unwrap();

        *context.state_change = GameStateChange::Swap(Box::new(State::default()));
    }
}

#[derive(Default)]
struct State {
    skeleton: Option<SpineSkeleton>,
}

impl GameState for State {
    fn enter(&mut self, context: GameContext) {
        let asset = context
            .assets
            .find("spine://robot.zip")
            .unwrap()
            .access::<&SpineAsset>(context.assets);

        let skeleton = SpineSkeleton::new(asset);
        skeleton.play_animation("idle", 0, 0.75, true).unwrap();
        self.skeleton = Some(skeleton);
    }

    fn fixed_update(&mut self, _: GameContext, delta_time: f32) {
        let Some(skeleton) = self.skeleton.as_ref() else {
            return;
        };
        skeleton.update(delta_time);
    }

    fn draw(&mut self, context: GameContext) {
        let Some(skeleton) = self.skeleton.as_ref() else {
            return;
        };
        skeleton.draw(context.draw, context.graphics);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    GameLauncher::new(GameInstance::new(Preloader).setup_assets(|assets| {
        *assets = make_directory_database("./resources/").unwrap();
    }))
    .title("Spine 2D")
    .config(Config::load_from_file("./resources/GameConfig.toml")?)
    .run();
    Ok(())
}
