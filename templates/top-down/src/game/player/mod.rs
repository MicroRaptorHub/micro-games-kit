pub mod conditions;
pub mod tasks;

use self::{
    conditions::{PlayerHasActiveWeapon, PlayerIsAttackingCondition, PlayerIsMovingCondition},
    tasks::{
        attack_axe::PlayerAttackAxeTask, attack_bow::PlayerAttackBowTask,
        attack_sword::PlayerAttackSwordTask, idle::PlayerIdleTask, run::PlayerRunTask,
    },
};
use super::{
    animation::Animation,
    character::{Character, CharacterController},
    game_object::GameObject,
};
use micro_games_kit::{
    context::GameContext,
    third_party::{
        emergent::builders::behavior_tree::BehaviorTree,
        spitfire_draw::{
            sprite::{Sprite, SpriteTexture},
            utils::{Drawable, ShaderRef, TextureRef},
        },
        spitfire_input::{CardinalInputCombinator, InputActionRef, InputMapping, VirtualAction},
        windowing::event::VirtualKeyCode,
    },
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlayerWeapon {
    #[default]
    Bow,
    Sword,
    Axe,
}

impl PlayerWeapon {
    pub fn prev(self) -> Self {
        match self {
            Self::Bow => Self::Axe,
            Self::Sword => Self::Bow,
            Self::Axe => Self::Sword,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Bow => Self::Sword,
            Self::Sword => Self::Axe,
            Self::Axe => Self::Bow,
        }
    }
}

#[derive(Default)]
pub struct PlayerInputState {
    pub movement: CardinalInputCombinator,
    pub attack: InputActionRef,
    pub weapon_prev: InputActionRef,
    pub weapon_next: InputActionRef,
}

pub struct PlayerState {
    pub sprite: Sprite,
    pub input: PlayerInputState,
    pub weapon: PlayerWeapon,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            sprite: Sprite::single(SpriteTexture::new(
                "u_image".into(),
                TextureRef::name("idle"),
            ))
            .shader(ShaderRef::name("image"))
            .pivot(0.5.into()),
            input: Default::default(),
            weapon: Default::default(),
        }
    }
}

impl GameObject for PlayerState {
    fn update(&mut self, _: &mut GameContext, _: f32) {
        if !self.input.weapon_prev.get().is_released() {
            self.weapon = self.weapon.prev();
        }
        if !self.input.weapon_next.get().is_released() {
            self.weapon = self.weapon.next();
        }
    }

    fn draw(&mut self, context: &mut GameContext) {
        self.sprite.draw(context.draw, context.graphics);
    }
}

impl PlayerState {
    pub fn new_character() -> Character<PlayerState> {
        let left = InputActionRef::default();
        let right = InputActionRef::default();
        let up = InputActionRef::default();
        let down = InputActionRef::default();

        let mut state = PlayerState::default();
        state.input.movement =
            CardinalInputCombinator::new(left.clone(), right.clone(), up.clone(), down.clone());

        let mapping = InputMapping::default()
            .action(VirtualAction::KeyButton(VirtualKeyCode::A), left)
            .action(VirtualAction::KeyButton(VirtualKeyCode::D), right)
            .action(VirtualAction::KeyButton(VirtualKeyCode::W), up)
            .action(VirtualAction::KeyButton(VirtualKeyCode::S), down)
            .action(
                VirtualAction::KeyButton(VirtualKeyCode::Space),
                state.input.attack.clone(),
            )
            .action(
                VirtualAction::KeyButton(VirtualKeyCode::Q),
                state.input.weapon_prev.clone(),
            )
            .action(
                VirtualAction::KeyButton(VirtualKeyCode::E),
                state.input.weapon_next.clone(),
            );

        let task = BehaviorTree::selector(true)
            .node(
                BehaviorTree::selector(PlayerIsAttackingCondition)
                    .node(BehaviorTree::state(
                        PlayerHasActiveWeapon(PlayerWeapon::Bow),
                        PlayerAttackBowTask::default(),
                    ))
                    .node(BehaviorTree::state(
                        PlayerHasActiveWeapon(PlayerWeapon::Sword),
                        PlayerAttackSwordTask::default(),
                    ))
                    .node(BehaviorTree::state(
                        PlayerHasActiveWeapon(PlayerWeapon::Axe),
                        PlayerAttackAxeTask::default(),
                    )),
            )
            .node(BehaviorTree::state(
                PlayerIsMovingCondition,
                PlayerRunTask::default(),
            ))
            .node(BehaviorTree::state(true, PlayerIdleTask::default()))
            .build();

        let controller = CharacterController::input(mapping);
        Character::new(state, task, controller)
    }

    pub fn apply_animation(&mut self, animation: &Animation) {
        if let Some(frame) = animation.current_frame() {
            self.sprite.textures[0].texture =
                TextureRef::name(format!("{}/{}", animation.id, frame));
        }
    }
}
