use micro_games_kit::{
    animation::spine::{AnimationPlayer, Skeleton},
    assets::{
        atlas::AtlasAsset, make_directory_database, shader::ShaderAsset, spine::SpineDocument,
    },
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

        context.assets.ensure("spine://robot.json").unwrap();
        context.assets.ensure("atlas://robot.atlas").unwrap();
        context.assets.ensure("texture://robot.png").unwrap();

        *context.state_change = GameStateChange::Swap(Box::new(State::default()));
    }
}

#[derive(Default)]
struct State {
    skeleton: Option<Skeleton>,
    anim_player: Option<AnimationPlayer>,
}

impl GameState for State {
    fn enter(&mut self, context: GameContext) {
        let document = context.assets.find("spine://robot.json").unwrap();
        let document = document.access::<&SpineDocument>(context.assets);
        let atlas = context.assets.find("atlas://robot.atlas").unwrap();
        let atlas = atlas.access::<&AtlasAsset>(context.assets);

        self.skeleton = Some(Skeleton::new(document, atlas, "default", "u_image").unwrap());
        self.anim_player = Some(AnimationPlayer::new(document).looped().playing("idle"));
    }

    fn fixed_update(&mut self, _: GameContext, delta_time: f32) {
        let Some(anim_player) = self.anim_player.as_mut() else {
            return;
        };
        let Some(skeleton) = self.skeleton.as_mut() else {
            return;
        };
        anim_player.update(delta_time);
        anim_player.apply_to_skeleton(skeleton);
    }

    fn draw(&mut self, context: GameContext) {
        let Some(skeleton) = self.skeleton.as_mut() else {
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
