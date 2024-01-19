use crate::game::ui::{make_theme, text_button::text_button};
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    third_party::{
        raui_immediate::apply_shared_props,
        raui_immediate_widgets::{
            core::{
                containers::{horizontal_box, nav_vertical_box},
                image_box, FlexBoxItemLayout, ImageBoxAspectRatio, ImageBoxImage, ImageBoxMaterial,
                ImageBoxProps, TextBoxVerticalAlign,
            },
            material::{text_paper, TextPaperProps},
        },
    },
};

use super::{gameplay::Gameplay, main_menu::MainMenu};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEndReason {
    Lost,
    Won,
}

impl ToString for GameEndReason {
    fn to_string(&self) -> String {
        match self {
            Self::Lost => "YOU LOST".to_owned(),
            Self::Won => "YOU WON".to_owned(),
        }
    }
}

pub struct GameEnd {
    reason: GameEndReason,
}

impl GameEnd {
    pub fn new(reason: GameEndReason) -> Self {
        Self { reason }
    }
}

impl GameState for GameEnd {
    fn enter(&mut self, context: GameContext) {
        context.graphics.color = [0.2, 0.2, 0.2, 1.0];
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
                    id: match self.reason {
                        GameEndReason::Lost => "ui/lost".to_owned(),
                        GameEndReason::Won => "ui/won".to_owned(),
                    },
                    ..Default::default()
                }),
                ..Default::default()
            });

            nav_vertical_box((), || {
                text_paper(TextPaperProps {
                    text: self.reason.to_string(),
                    variant: "title".to_owned(),
                    vertical_align_override: Some(TextBoxVerticalAlign::Top),
                    color_override: Some(Default::default()),
                    ..Default::default()
                });

                horizontal_box(
                    FlexBoxItemLayout {
                        basis: Some(100.0),
                        grow: 0.0,
                        shrink: 0.0,
                        ..Default::default()
                    },
                    || {
                        let restart = text_button(
                            FlexBoxItemLayout {
                                margin: 20.0.into(),
                                ..Default::default()
                            },
                            "Restart",
                        );

                        let exit = text_button(
                            FlexBoxItemLayout {
                                margin: 20.0.into(),
                                ..Default::default()
                            },
                            "Exit",
                        );

                        if exit.trigger_stop() {
                            *context.state_change = GameStateChange::Swap(Box::new(MainMenu));
                        } else if restart.trigger_stop() {
                            *context.state_change =
                                GameStateChange::Swap(Box::<Gameplay>::default());
                        }
                    },
                );
            });
        });
    }
}
