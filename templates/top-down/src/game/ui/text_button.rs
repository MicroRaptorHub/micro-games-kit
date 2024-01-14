use micro_games_kit::third_party::{
    raui_core::props::Props,
    raui_immediate_widgets::core::{
        containers::content_box,
        image_box,
        interactive::{button, ImmediateButton, NavItemActive},
        text_box, Color, ContentBoxItemLayout, ImageBoxProps, TextBoxFont, TextBoxHorizontalAlign,
        TextBoxProps, TextBoxVerticalAlign,
    },
};

pub fn text_button(props: impl Into<Props>, message: impl ToString) -> ImmediateButton {
    button(props.into().with(NavItemActive), |state| {
        content_box((), || {
            let (image_color, text_color) = if state.state.trigger {
                (
                    Color {
                        r: 1.0,
                        g: 0.6,
                        b: 0.1,
                        a: 1.0,
                    },
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                )
            } else if state.state.selected {
                (
                    Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                )
            } else {
                (
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                )
            };

            image_box(ImageBoxProps::colored(image_color));

            text_box((
                ContentBoxItemLayout {
                    margin: 10.0.into(),
                    ..Default::default()
                },
                TextBoxProps {
                    text: message.to_string(),
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    vertical_align: TextBoxVerticalAlign::Middle,
                    font: TextBoxFont {
                        name: "roboto".to_owned(),
                        size: 38.0,
                    },
                    color: text_color,
                    ..Default::default()
                },
            ));
        });
    })
}
