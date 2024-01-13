use micro_games_kit::{
    context::GameContext,
    game_object::GameObject,
    third_party::{
        rand::random,
        spitfire_draw::{
            sprite::{Sprite, SpriteTexture},
            utils::{Drawable, TextureRef},
        },
        spitfire_glow::renderer::GlowTextureFiltering,
        vek::Vec2,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Apple,
    Orange,
    Banana,
}

impl ItemKind {
    pub fn random() -> Self {
        let index = random::<usize>() % 3;
        match index {
            0 => Self::Apple,
            1 => Self::Orange,
            2 => Self::Banana,
            _ => unreachable!(),
        }
    }

    pub fn texture(self) -> &'static str {
        match self {
            Self::Apple => "item/apple",
            Self::Orange => "item/orange",
            Self::Banana => "item/banana",
        }
    }

    pub fn radius(self) -> f32 {
        20.0
    }

    pub fn health(self) -> usize {
        match self {
            Self::Apple => 10,
            Self::Orange => 0,
            Self::Banana => 5,
        }
    }

    pub fn attack(self) -> usize {
        match self {
            Self::Apple => 0,
            Self::Orange => 10,
            Self::Banana => 5,
        }
    }
}

pub struct Item {
    pub sprite: Sprite,
    pub collision_radius: f32,
    pub health: usize,
    pub attack: usize,
}

impl Item {
    pub fn new(kind: ItemKind, position: impl Into<Vec2<f32>>) -> Self {
        Self {
            sprite: Sprite::single(SpriteTexture {
                sampler: "u_image".into(),
                texture: TextureRef::name(kind.texture()),
                filtering: GlowTextureFiltering::Linear,
            })
            .position(position.into()),
            collision_radius: kind.radius(),
            health: kind.health(),
            attack: kind.attack(),
        }
    }

    pub fn does_collide(&self, position: Vec2<f32>) -> bool {
        self.sprite.transform.position.xy().distance(position) <= self.collision_radius
    }
}

impl GameObject for Item {
    fn draw(&mut self, context: &mut GameContext) {
        self.sprite.draw(context.draw, context.graphics);
    }
}
