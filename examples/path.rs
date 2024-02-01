use anim8::{
    spline::Spline,
    utils::{factor_iter, range_iter},
};
use micro_games_kit::{
    config::Config,
    context::GameContext,
    game::{GameInstance, GameState},
    loader::load_shader,
    GameLauncher,
};
use spitfire_draw::{
    primitives::PrimitivesEmitter,
    utils::{Drawable, ShaderRef},
};
use spitfire_glow::graphics::{CameraScaling, Shader};
use std::error::Error;
use vek::Rgba;

struct State {
    spline: Spline<[f32; 2]>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            spline: Spline::smooth(
                &[
                    [-100.0, -100.0],
                    [100.0, -100.0],
                    [-100.0, 100.0],
                    [100.0, 100.0],
                ],
                1.0,
            )
            .unwrap(),
        }
    }
}

impl GameState for State {
    fn enter(&mut self, context: GameContext) {
        context.graphics.color = [0.2, 0.2, 0.2, 1.0];
        context.graphics.main_camera.screen_alignment = 0.5.into();
        context.graphics.main_camera.scaling = CameraScaling::FitVertical(300.0);

        load_shader(
            context.draw,
            context.graphics,
            "color",
            Shader::COLORED_VERTEX_2D,
            Shader::PASS_FRAGMENT,
        );
    }

    fn draw(&mut self, context: GameContext) {
        let emitter = PrimitivesEmitter::default().shader(ShaderRef::name("color"));

        emitter
            .emit_lines(factor_iter(50).map(|factor| self.spline.sample(factor).into()))
            .thickness(2.0)
            .tint(Rgba::new(0.25, 0.25, 1.0, 1.0))
            .draw(context.draw, context.graphics);

        for distance in range_iter(30, 0.0, self.spline.length()) {
            let factor = self.spline.find_time_for_distance(distance);
            let point = self.spline.sample(factor);
            emitter
                .emit_circle(point.into(), 1.0, 0.1)
                .tint(Rgba::new(0.25, 1.0, 0.25, 1.0))
                .draw(context.draw, context.graphics);
        }

        for point in self.spline.points() {
            emitter
                .emit_circle(point.point.into(), 2.0, 0.1)
                .tint(Rgba::new(1.0, 0.25, 0.25, 1.0))
                .draw(context.draw, context.graphics);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    GameLauncher::new(GameInstance::new(State::default()))
        .title("Path")
        .config(Config::load_from_file("./resources/GameConfig.toml")?)
        .run();
    Ok(())
}
