use super::utils::world_to_screen_anchor;
use micro_games_kit::{
    context::GameContext,
    third_party::{
        raui_immediate_widgets::core::{ContentBoxItemLayout, Rect},
        vek::Vec2,
    },
};

pub mod health_bar;

pub fn world_to_screen_content_layout(
    position: Vec2<f32>,
    region: Rect,
    context: &GameContext,
) -> ContentBoxItemLayout {
    let anchor = world_to_screen_anchor(position, context);
    ContentBoxItemLayout {
        anchors: Rect {
            left: anchor.x,
            right: anchor.x,
            top: anchor.y,
            bottom: anchor.y,
        },
        margin: Rect {
            left: region.left,
            right: -region.right,
            top: region.top,
            bottom: -region.bottom,
        },
        ..Default::default()
    }
}
