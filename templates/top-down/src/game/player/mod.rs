pub mod conditions;
pub mod tasks;

use self::{
    conditions::{PlayerHasActiveWeapon, PlayerIsAttackingCondition, PlayerIsMovingCondition},
    tasks::{
        attack_axe::PlayerAttackAxeTask, attack_sword::PlayerAttackSwordTask, idle::PlayerIdleTask,
        run::PlayerRunTask,
    },
};
use micro_games_kit::{
    animation::NamedAnimation,
    character::{Character, CharacterController},
    context::GameContext,
    game_object::GameObject,
    third_party::{
        emergent::builders::behavior_tree::BehaviorTree,
        spitfire_draw::{
            sprite::{Sprite, SpriteTexture},
            utils::{Drawable, ShaderRef, TextureRef},
        },
        spitfire_glow::renderer::GlowUniformValue,
        spitfire_input::{CardinalInputCombinator, InputActionRef, InputMapping, VirtualAction},
        windowing::event::VirtualKeyCode,
    },
};

use super::utils::events::{Event, Events, Instigator};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlayerWeapon {
    #[default]
    Sword,
    Axe,
}

impl PlayerWeapon {
    pub fn prev(self) -> Self {
        match self {
            Self::Sword => Self::Axe,
            Self::Axe => Self::Sword,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Sword => Self::Axe,
            Self::Axe => Self::Sword,
        }
    }

    pub fn attack(self) -> usize {
        match self {
            Self::Sword => 25,
            Self::Axe => 40,
        }
    }

    pub fn range(self) -> f32 {
        match self {
            Self::Sword => 30.0,
            Self::Axe => 50.0,
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
    pub health: usize,
    pub blink_seconds: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            sprite: Sprite::single(SpriteTexture::new(
                "u_image".into(),
                TextureRef::name("player/idle/1"),
            ))
            .shader(ShaderRef::name("character"))
            .pivot(0.5.into())
            .uniform(
                "u_fill_color".into(),
                GlowUniformValue::F4([0.0, 0.0, 0.0, 0.0]),
            ),
            input: Default::default(),
            weapon: Default::default(),
            health: 100,
            blink_seconds: 0.0,
        }
    }
}

impl GameObject for PlayerState {
    fn update(&mut self, _: &mut GameContext, delta_time: f32) {
        if self.input.weapon_prev.get().is_pressed() {
            self.weapon = self.weapon.prev();
        }
        if self.input.weapon_next.get().is_pressed() {
            self.weapon = self.weapon.next();
        }
        self.blink_seconds = (self.blink_seconds - delta_time).max(0.0);
    }

    fn draw(&mut self, context: &mut GameContext) {
        if self.blink_seconds > 0.0 {
            self.sprite.uniforms.insert(
                "u_fill_color".into(),
                GlowUniformValue::F4([1.0, 1.0, 1.0, 1.0]),
            );
        } else {
            self.sprite.uniforms.insert(
                "u_fill_color".into(),
                GlowUniformValue::F4([1.0, 1.0, 1.0, 0.0]),
            );
        }
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

    pub fn apply_animation(&mut self, animation: &NamedAnimation) {
        if let Some(frame) = animation.animation.current_frame() {
            self.sprite.textures[0].texture =
                TextureRef::name(format!("{}/{}", animation.id, frame));
        }
    }

    pub fn execute_events(&mut self, events: &[Event]) {
        for event in events {
            if let Event::Attack {
                position,
                range,
                value,
                instigator,
            } = event
            {
                if *instigator == Instigator::Enemy {
                    let distance = position.distance(self.sprite.transform.position.xy());
                    if distance <= *range {
                        self.blink_seconds = 0.15;
                        self.health = self.health.saturating_sub(*value);
                        if self.health == 0 {
                            Events::write(Event::KillPlayer);
                        }
                    }
                }
            }
        }
    }
}
