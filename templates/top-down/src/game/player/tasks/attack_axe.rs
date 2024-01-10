use crate::game::{animation::Animation, character::CharacterMemory, player::PlayerState};
use micro_games_kit::third_party::emergent::task::Task;

#[derive(Debug, Clone)]
pub struct PlayerAttackAxeTask {
    animation: Animation,
}

impl Default for PlayerAttackAxeTask {
    fn default() -> Self {
        Self {
            animation: Animation::new("axe", 1..9),
        }
    }
}

impl Task<CharacterMemory<PlayerState>> for PlayerAttackAxeTask {
    fn is_locked(&self, _: &CharacterMemory<PlayerState>) -> bool {
        self.animation.is_playing
    }

    fn on_enter(&mut self, _: &mut CharacterMemory<PlayerState>) {
        self.animation.play();
    }

    fn on_exit(&mut self, _: &mut CharacterMemory<PlayerState>) {
        self.animation.stop();
    }

    fn on_update(&mut self, memory: &mut CharacterMemory<PlayerState>) {
        self.animation.update(memory.delta_time);

        memory
            .state
            .write()
            .unwrap()
            .apply_animation(&self.animation);
    }
}
