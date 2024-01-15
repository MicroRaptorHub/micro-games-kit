use super::gameplay::Gameplay;
use crate::game::ui::text_button::text_button;
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    third_party::raui_immediate_widgets::core::{
        containers::nav_vertical_box, text_box, Color, FlexBoxItemLayout, TextBoxFont,
        TextBoxHorizontalAlign, TextBoxProps, TextBoxVerticalAlign,
    },
};

pub struct MainMenu;

impl GameState for MainMenu {
    fn enter(&mut self, context: GameContext) {
        context.graphics.color = [0.2, 0.2, 0.2];
    }

    fn draw_gui(&mut self, context: GameContext) {
        nav_vertical_box((), || {
            let button_props = FlexBoxItemLayout {
                basis: Some(60.0),
                grow: 0.0,
                shrink: 0.0,
                margin: 20.0.into(),
                ..Default::default()
            };

            text_box(TextBoxProps {
                text: "TOP-DOWN GAME".to_owned(),
                horizontal_align: TextBoxHorizontalAlign::Center,
                vertical_align: TextBoxVerticalAlign::Middle,
                font: TextBoxFont {
                    name: "roboto".to_owned(),
                    size: 100.0,
                },
                color: Color {
                    r: 1.0,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
                },
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
    }
}
