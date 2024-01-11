pub mod ai;
pub mod conditions;
pub mod tasks;

use self::{
    ai::{pursue::EnemyAiPursueTask, wait::EnemyAiWaitTask, wander::EnemyAiWanderTask},
    conditions::{
        EnemyHasPlayerInRangeCondition, EnemyIsAttackingCondition, EnemyIsMovingCondition,
        EnemyIsNotInCooldownCondition,
    },
    tasks::{attack::EnemyAttackTask, idle::EnemyIdleTask, run::EnemyRunTask},
};
use super::player::PlayerState;
use micro_games_kit::{
    animation::NamedAnimation,
    character::{Character, CharacterController},
    context::GameContext,
    game_object::GameObject,
    third_party::{
        emergent::{builders::behavior_tree::BehaviorTree, combinators::all::CombinatorAll},
        spitfire_draw::{
            sprite::{Sprite, SpriteTexture},
            utils::{Drawable, ShaderRef, TextureRef},
        },
        vek::{Vec2, Vec3},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyAiBehavior {
    Wait,
    Wander,
    Pursue,
}

#[derive(Debug, Default)]
pub struct EnemyAiState {
    pub direction: Vec2<f32>,
    pub attack: bool,
    pub cooldown_seconds: f32,
    pub player_in_range_distance_threshold: f32,
    pub player_in_range_position: Option<Vec2<f32>>,
    pub player_target_offset_x: f32,
}

pub struct EnemyState {
    pub sprite: Sprite,
    pub ai: EnemyAiState,
}

impl Default for EnemyState {
    fn default() -> Self {
        Self {
            sprite: Sprite::single(SpriteTexture::new(
                "u_image".into(),
                TextureRef::name("idle"),
            ))
            .shader(ShaderRef::name("image"))
            .pivot([0.25, 0.5].into()),
            ai: Default::default(),
        }
    }
}

impl GameObject for EnemyState {
    fn update(&mut self, _: &mut GameContext, delta_time: f32) {
        self.ai.cooldown_seconds = (self.ai.cooldown_seconds - delta_time).max(0.0);
    }

    fn draw(&mut self, context: &mut GameContext) {
        self.sprite.draw(context.draw, context.graphics);
    }
}

impl EnemyState {
    pub fn new_character(position: impl Into<Vec3<f32>>) -> Character<EnemyState> {
        let mut state = EnemyState::default();
        state.ai.player_in_range_distance_threshold = 150.0;
        state.ai.player_target_offset_x = 20.0;
        state.sprite.transform.position = position.into();

        let task = BehaviorTree::selector(true)
            .node(BehaviorTree::state(
                EnemyIsAttackingCondition,
                EnemyAttackTask::default(),
            ))
            .node(BehaviorTree::state(
                EnemyIsMovingCondition,
                EnemyRunTask::default(),
            ))
            .node(BehaviorTree::state(true, EnemyIdleTask::default()))
            .build();

        let ai = BehaviorTree::selector(true)
            .node(BehaviorTree::state(
                CombinatorAll::default()
                    .condition(EnemyIsNotInCooldownCondition)
                    .condition(EnemyHasPlayerInRangeCondition),
                EnemyAiPursueTask::default(),
            ))
            .node(BehaviorTree::state(
                EnemyIsNotInCooldownCondition,
                EnemyAiWanderTask::default(),
            ))
            .node(BehaviorTree::state(true, EnemyAiWaitTask))
            .build();

        let controller = CharacterController::ai(ai);
        Character::new(state, task, controller)
    }

    pub fn apply_animation(&mut self, animation: &NamedAnimation) {
        if let Some(frame) = animation.animation.current_frame() {
            self.sprite.textures[0].texture =
                TextureRef::name(format!("{}/{}", animation.id, frame));
        }
    }

    pub fn sense_player(&mut self, player: &PlayerState) {
        let direction = player.sprite.transform.position.xy() - self.sprite.transform.position.xy();

        if direction.magnitude() < self.ai.player_in_range_distance_threshold {
            self.ai.player_in_range_position = Some(player.sprite.transform.position.xy());
        } else {
            self.ai.player_in_range_position = None;
        }
    }
}
