use micro_games_kit::third_party::{
    raui_core::props::Props,
    raui_immediate_widgets::core::{
        containers::content_box, image_box, Color, ContentBoxItemLayout, ImageBoxProps, Rect,
    },
};

pub fn health_bar(props: impl Into<Props>, value: usize) {
    let percentage = (value as f32 / 100.0).clamp(0.0, 1.0);

    content_box(props, || {
        image_box(ImageBoxProps::colored(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }));

        image_box((
            ContentBoxItemLayout {
                anchors: Rect {
                    left: 0.0,
                    right: percentage,
                    top: 0.0,
                    bottom: 1.0,
                },
                margin: 4.0.into(),
                ..Default::default()
            },
            ImageBoxProps::colored(if percentage < 0.25 {
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                }
            }),
        ));
    });
}
