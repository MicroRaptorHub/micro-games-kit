use crate::game::ui::text_button::text_button;
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    third_party::raui_immediate_widgets::core::{
        containers::{horizontal_box, nav_vertical_box},
        text_box, Color, FlexBoxItemLayout, TextBoxFont, TextBoxHorizontalAlign, TextBoxProps,
        TextBoxVerticalAlign,
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
        context.graphics.color = [0.2, 0.2, 0.2];
    }

    fn draw_gui(&mut self, context: GameContext) {
        nav_vertical_box((), || {
            text_box(TextBoxProps {
                text: self.reason.to_string(),
                horizontal_align: TextBoxHorizontalAlign::Center,
                vertical_align: TextBoxVerticalAlign::Middle,
                font: TextBoxFont {
                    name: "roboto".to_owned(),
                    size: 150.0,
                },
                color: Color {
                    r: 1.0,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
                },
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
                        *context.state_change = GameStateChange::Swap(Box::<Gameplay>::default());
                    }
                },
            );
        });
    }
}
