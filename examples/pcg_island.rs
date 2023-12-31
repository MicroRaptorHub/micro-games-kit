use micro_games_kit::{
    config::Config,
    context::GameContext,
    game::{GameInstance, GameState},
    grid_world::{GridWorld, GridWorldLayer},
    loader::load_shader,
    pcg::{Grid, NoiseGenerator, RemapGenerator, SubGenerator},
    GameLauncher,
};
use noise::{Fbm, MultiFractal, SuperSimplex};
use spitfire_draw::{
    tiles::{TileMap, TileSet, TileSetItem},
    utils::{Drawable, ShaderRef},
};
use spitfire_glow::graphics::{CameraScaling, Shader};
use std::error::Error;
use vek::{Rgba, Vec2};

const SIZE: usize = 50;
const WATER: usize = 0;
const FOREST: usize = 1;
const GRASS: usize = 2;
const SAND: usize = 3;
const ROCK: usize = 4;
const SNOW: usize = 5;

struct State {
    world: GridWorld,
}

impl Default for State {
    fn default() -> Self {
        let mut height = Grid::<f64>::generate(
            SIZE.into(),
            NoiseGenerator::new(Fbm::<SuperSimplex>::default().set_frequency(0.025)),
        );
        height.apply_all(RemapGenerator {
            from: -1.0..1.0,
            to: 0.0..1.0,
        });

        let gradient = Grid::<f64>::generate(
            SIZE.into(),
            |location: Vec2<usize>, size: Vec2<usize>, _| {
                let center = size / 2;
                let x = if location.x >= center.x {
                    location.x - center.x
                } else {
                    center.x - location.x
                } as f64;
                let y = if location.y >= center.y {
                    location.y - center.y
                } else {
                    center.y - location.y
                } as f64;
                let result = (x / center.x as f64).max(y / center.y as f64);
                result * result
            },
        );
        height.apply_all(SubGenerator { other: &gradient });

        let mut biome = Grid::<f64>::generate(
            SIZE.into(),
            NoiseGenerator::new(Fbm::<SuperSimplex>::new(42).set_frequency(0.05)),
        );
        biome.apply_all(RemapGenerator {
            from: -1.0..1.0,
            to: 0.0..1.0,
        });

        let buffer = height
            .into_inner()
            .1
            .into_iter()
            .zip(biome.into_inner().1)
            .map(|(height, biome)| {
                if height > 0.75 {
                    SNOW
                } else if height > 0.6 {
                    ROCK
                } else if height > 0.1 {
                    if biome > 0.8 {
                        SAND
                    } else if biome > 0.5 {
                        GRASS
                    } else {
                        FOREST
                    }
                } else {
                    WATER
                }
            })
            .collect();

        Self {
            world: GridWorld::new(
                10.0.into(),
                TileSet::default()
                    .shader(ShaderRef::name("color"))
                    .mapping(WATER, TileSetItem::default().tint(Rgba::blue()))
                    .mapping(
                        FOREST,
                        TileSetItem::default().tint(Rgba::new_opaque(0.0, 0.5, 0.0)),
                    )
                    .mapping(GRASS, TileSetItem::default().tint(Rgba::green()))
                    .mapping(
                        SAND,
                        TileSetItem::default().tint(Rgba::new_opaque(1.0, 1.0, 0.5)),
                    )
                    .mapping(ROCK, TileSetItem::default().tint(Rgba::gray(0.5)))
                    .mapping(SNOW, TileSetItem::default().tint(Rgba::white())),
                GridWorldLayer::new(TileMap::with_buffer(SIZE.into(), buffer).unwrap()),
            ),
        }
    }
}

impl GameState for State {
    fn enter(&mut self, context: GameContext) {
        context.graphics.main_camera.scaling = CameraScaling::FitVertical(SIZE as f32 * 10.0);

        load_shader(
            context.draw,
            context.graphics,
            "color",
            Shader::COLORED_VERTEX_2D,
            Shader::PASS_FRAGMENT,
        );
    }

    fn draw(&mut self, context: GameContext) {
        self.world.draw(context.draw, context.graphics);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    GameLauncher::new(GameInstance::new(State::default()))
        .title("Procedural Content Generator - Island")
        .config(Config::load_from_file("./resources/GameConfig.toml")?)
        .run();
    Ok(())
}
