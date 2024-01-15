use crate::game::{player::PlayerState, utils::audio::Audio};
use micro_games_kit::{
    animation::{FrameAnimation, NamedAnimation},
    character::CharacterMemory,
    third_party::{
        emergent::task::Task,
        rand::{thread_rng, Rng},
        vek::Vec3,
    },
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
                animation: FrameAnimation::new(1..25)
                    .looping()
                    .event(6, "footstep")
                    .event(18, "footstep"),
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

        let events = self.animation.animation.update(memory.delta_time);
        {
            let mut audio = Audio::write();
            let mut audio = audio.write().unwrap();
            for event in events {
                if event == "footstep" {
                    audio.play(match thread_rng().gen_range(1..=3) {
                        1 => "footstep/grass/1",
                        2 => "footstep/grass/2",
                        3 => "footstep/grass/3",
                        _ => unreachable!(),
                    });
                }
            }
        }
        state.apply_animation(&self.animation);

        state.sprite.transform.position += direction * self.speed * memory.delta_time;
        if x > 0.1 {
            state.sprite.transform.scale.x = -1.0;
        } else if x < -0.1 {
            state.sprite.transform.scale.x = 1.0;
        }
    }
}
