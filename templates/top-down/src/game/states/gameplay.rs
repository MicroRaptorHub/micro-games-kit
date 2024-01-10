use crate::game::{character::Character, game_object::GameObject, player::PlayerState};
use micro_games_kit::{
    context::GameContext,
    game::{GameState, GameStateChange},
    third_party::{
        raui_core::layout::CoordsMappingScaling,
        spitfire_glow::graphics::CameraScaling,
        spitfire_input::{InputActionRef, InputConsume, InputMapping, VirtualAction},
        typid::ID,
        windowing::event::VirtualKeyCode,
    },
};

pub struct Gameplay {
    player: Character<PlayerState>,
    exit: InputActionRef,
    exit_handle: Option<ID<InputMapping>>,
}

impl Default for Gameplay {
    fn default() -> Self {
        Self {
            player: PlayerState::new_character(),
            exit: Default::default(),
            exit_handle: None,
        }
    }
}

impl GameState for Gameplay {
    fn enter(&mut self, mut context: GameContext) {
        context.graphics.color = [0.0, 0.3, 0.0];
        context.graphics.main_camera.screen_alignment = 0.5.into();
        context.graphics.main_camera.scaling = CameraScaling::FitVertical(300.0);
        context.gui.coords_map_scaling = CoordsMappingScaling::FitVertical(1024.0);

        self.exit_handle = Some(context.input.push_mapping(
            InputMapping::default().consume(InputConsume::Hit).action(
                VirtualAction::KeyButton(VirtualKeyCode::Escape),
                self.exit.clone(),
            ),
        ));

        self.player.activate(&mut context);
    }

    fn exit(&mut self, mut context: GameContext) {
        self.player.deactivate(&mut context);

        if let Some(id) = self.exit_handle {
            context.input.remove_mapping(id);
            self.exit_handle = None;
        }
    }

    fn update(&mut self, mut context: GameContext, delta_time: f32) {
        if self.exit.get().is_down() {
            *context.state_change = GameStateChange::Pop;
        }

        self.player.update(&mut context, delta_time);
    }
    fn draw(&mut self, mut context: GameContext) {
        self.player.draw(&mut context);
    }
}
