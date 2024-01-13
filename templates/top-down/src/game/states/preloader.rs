use super::gameplay::Gameplay;
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    loader::{load_font, load_shader, load_texture},
    third_party::spitfire_glow::graphics::Shader,
};

macro_rules! load_texture_series {
    (
        $context:expr,
        $id:literal,
        [$($index:literal),+]
    ) => {
        $(
            load_texture(
                $context.draw,
                $context.graphics,
                concat!($id, "/", $index),
                include_bytes!(concat!(
                    "../../../assets/images/", $id, "-", $index, ".png"),
                ),
                1,
                1,
            );
        )+
    };
}

pub struct Preloader;

impl GameState for Preloader {
    fn enter(&mut self, mut context: GameContext) {
        Self::load_shaders(&mut context);
        Self::load_fonts(&mut context);
        Self::load_textures(&mut context);

        *context.state_change = GameStateChange::Swap(Box::<Gameplay>::default());
    }
}

impl Preloader {
    fn load_shaders(context: &mut GameContext) {
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
        load_shader(
            context.draw,
            context.graphics,
            "character",
            Shader::TEXTURED_VERTEX_2D,
            include_str!("../../../assets/shaders/character.glsl"),
        );
    }

    fn load_fonts(context: &mut GameContext) {
        load_font(
            context.draw,
            "roboto",
            include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"),
        );
    }

    fn load_textures(context: &mut GameContext) {
        // player character
        load_texture_series!(context, "player/idle", [1]);
        load_texture_series!(
            context,
            "player/run",
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24
            ]
        );
        load_texture_series!(context, "player/axe", [1, 2, 3, 4, 5, 6, 7, 8]);
        load_texture_series!(context, "player/sword", [1, 2, 3, 4, 5, 6, 7]);

        // enemy character
        load_texture_series!(context, "enemy/idle", [1, 2, 3, 4, 5]);
        load_texture_series!(context, "enemy/run", [1, 2, 3, 4, 5, 6, 7, 8]);
        load_texture_series!(context, "enemy/attack", [1, 2, 3, 4, 5, 6, 7, 8]);

        // enviro
        load_texture_series!(context, "enviro/tree", [1, 2, 3, 4, 5, 6]);

        // items
        load_texture(
            context.draw,
            context.graphics,
            "item/apple",
            include_bytes!("../../../assets/images/item/apple.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "item/banana",
            include_bytes!("../../../assets/images/item/banana.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "item/orange",
            include_bytes!("../../../assets/images/item/orange.png"),
            1,
            1,
        );
    }
}