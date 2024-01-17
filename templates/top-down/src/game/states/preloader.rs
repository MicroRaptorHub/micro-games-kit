use super::main_menu::MainMenu;
use crate::game::utils::audio::Audio;
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    loader::{load_font, load_shader, load_texture},
    third_party::{
        spitfire_glow::graphics::Shader,
        spitfire_gui::interactions::GuiInteractionsInputs,
        spitfire_input::{
            ArrayInputCombinator, InputActionRef, InputAxisRef, InputConsume, InputMapping,
            VirtualAction, VirtualAxis,
        },
        windowing::event::MouseButton,
    },
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
        Self::load_sounds_and_music();
        Self::setup_gui_inputs(&mut context);

        *context.state_change = GameStateChange::Swap(Box::new(MainMenu));
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
        // map
        load_texture(
            context.draw,
            context.graphics,
            "map/level-0",
            include_bytes!("../../../assets/maps/world/simplified/Level_0/_composite.png"),
            1,
            1,
        );

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

        // ui
        load_texture(
            context.draw,
            context.graphics,
            "ui/panel",
            include_bytes!("../../../assets/images/ui/panel.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/bar",
            include_bytes!("../../../assets/images/ui/bar.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/button/idle",
            include_bytes!("../../../assets/images/ui/button-idle.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/button/select",
            include_bytes!("../../../assets/images/ui/button-select.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/button/trigger",
            include_bytes!("../../../assets/images/ui/button-trigger.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/cover",
            include_bytes!("../../../assets/images/ui/cover.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/won",
            include_bytes!("../../../assets/images/ui/won.png"),
            1,
            1,
        );
        load_texture(
            context.draw,
            context.graphics,
            "ui/lost",
            include_bytes!("../../../assets/images/ui/lost.png"),
            1,
            1,
        );
    }

    fn load_sounds_and_music() {
        let mut audio = Audio::write();
        let mut audio = audio.write().unwrap();

        audio.register(
            "footstep/grass/1",
            include_bytes!("../../../assets/sounds/footstep-grass-1.ogg"),
        );
        audio.register(
            "footstep/grass/2",
            include_bytes!("../../../assets/sounds/footstep-grass-2.ogg"),
        );
        audio.register(
            "footstep/grass/3",
            include_bytes!("../../../assets/sounds/footstep-grass-3.ogg"),
        );
        audio.register("sword", include_bytes!("../../../assets/sounds/sword.ogg"));
        audio.register("axe", include_bytes!("../../../assets/sounds/axe.ogg"));
        audio.register(
            "collect",
            include_bytes!("../../../assets/sounds/collect.ogg"),
        );
        audio.register(
            "button/select",
            include_bytes!("../../../assets/sounds/button-select.ogg"),
        );
        audio.register(
            "button/click",
            include_bytes!("../../../assets/sounds/button-click.ogg"),
        );

        audio.register("forest", include_bytes!("../../../assets/music/forest.ogg"));
        audio.register("battle", include_bytes!("../../../assets/music/battle.ogg"));
    }

    fn setup_gui_inputs(context: &mut GameContext) {
        context
            .gui
            .interactions
            .engine
            .deselect_when_no_button_found = true;

        let pointer_x = InputAxisRef::default();
        let pointer_y = InputAxisRef::default();
        let pointer_trigger = InputActionRef::default();

        context.gui.interactions.inputs = GuiInteractionsInputs {
            pointer_position: ArrayInputCombinator::new([pointer_x.clone(), pointer_y.clone()]),
            pointer_trigger: pointer_trigger.clone(),
            ..Default::default()
        };

        context.input.push_mapping(
            InputMapping::default()
                .consume(InputConsume::Hit)
                .axis(VirtualAxis::MousePositionX, pointer_x)
                .axis(VirtualAxis::MousePositionY, pointer_y)
                .action(
                    VirtualAction::MouseButton(MouseButton::Left),
                    pointer_trigger,
                ),
        );
    }
}
