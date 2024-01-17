use super::gameplay::Gameplay;
use crate::game::ui::{make_theme, text_button::text_button};
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    third_party::{
        raui_immediate::apply_shared_props,
        raui_immediate_widgets::{
            core::{
                containers::nav_vertical_box, image_box, FlexBoxItemLayout, ImageBoxAspectRatio,
                ImageBoxImage, ImageBoxMaterial, ImageBoxProps, TextBoxVerticalAlign,
            },
            material::{text_paper, TextPaperProps},
        },
    },
};

pub struct MainMenu;

impl GameState for MainMenu {
    fn enter(&mut self, context: GameContext) {
        context.graphics.color = [0.2, 0.2, 0.2];
        context.gui.coords_map_scaling = Default::default();
    }

    fn draw_gui(&mut self, context: GameContext) {
        apply_shared_props(make_theme(), || {
            image_box(ImageBoxProps {
                content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                    horizontal_alignment: 0.5,
                    vertical_alignment: 0.0,
                    outside: true,
                }),
                material: ImageBoxMaterial::Image(ImageBoxImage {
                    id: "ui/cover".to_owned(),
                    ..Default::default()
                }),
                ..Default::default()
            });

            nav_vertical_box((), || {
                let button_props = FlexBoxItemLayout {
                    basis: Some(60.0),
                    grow: 0.0,
                    shrink: 0.0,
                    margin: 20.0.into(),
                    ..Default::default()
                };

                text_paper(TextPaperProps {
                    text: "RED HOOD".to_owned(),
                    variant: "title".to_owned(),
                    vertical_align_override: Some(TextBoxVerticalAlign::Bottom),
                    color_override: Some(Default::default()),
                    ..Default::default()
                });

                let new_game = text_button(button_props.clone(), "New Game");
                let exit = text_button(button_props, "Exit");

                if new_game.trigger_stop() {
                    *context.state_change = GameStateChange::Swap(Box::<Gameplay>::default());
                } else if exit.trigger_stop() {
                    *context.state_change = GameStateChange::Pop;
                }
            });
        });
    }
}
