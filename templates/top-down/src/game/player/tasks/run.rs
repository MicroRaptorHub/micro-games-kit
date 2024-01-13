use crate::game::player::PlayerState;
use micro_games_kit::{
    animation::{FrameAnimation, NamedAnimation},
    character::CharacterMemory,
    third_party::{emergent::task::Task, vek::Vec3},
};

#[derive(Debug, Clone)]
pub struct PlayerRunTask {
    animation: NamedAnimation,
    speed: f32,
}

impl Default for PlayerRunTask {
    fn default() -> Self {
        Self {
            animation: NamedAnimation {
                animation: FrameAnimation::new(1..25).looping(),
                id: "player/run".to_owned(),
            },
            speed: 80.0,
        }
    }
}

impl Task<CharacterMemory<PlayerState>> for PlayerRunTask {
    fn on_enter(&mut self, _: &mut CharacterMemory<PlayerState>) {
        self.animation.animation.play();
    }

    fn on_exit(&mut self, _: &mut CharacterMemory<PlayerState>) {
        self.animation.animation.stop();
    }

    fn on_update(&mut self, memory: &mut CharacterMemory<PlayerState>) {
        let mut state = memory.state.write().unwrap();
        let [x, y] = state.input.movement.get();
        let direction = Vec3::new(x, y, 0.0).try_normalized().unwrap_or_default();

        self.animation.animation.update(memory.delta_time);
        state.apply_animation(&self.animation);

        state.sprite.transform.position += direction * self.speed * memory.delta_time;
        if x > 0.1 {
            state.sprite.transform.scale.x = -1.0;
        } else if x < -0.1 {
            state.sprite.transform.scale.x = 1.0;
        }
    }
}
