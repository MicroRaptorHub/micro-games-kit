use crate::game::{
    enemy::EnemyState,
    item::{Item, ItemKind},
    player::PlayerState,
    ui::{health_bar::health_bar, world_to_screen_content_layout},
    utils::events::{Event, Events},
};
use micro_games_kit::{
    character::Character,
    context::GameContext,
    game::{GameState, GameStateChange},
    game_object::GameObject,
    third_party::{
        rand::{thread_rng, Rng},
        raui_core::layout::CoordsMappingScaling,
        raui_immediate_widgets::core::{containers::content_box, Rect},
        spitfire_glow::graphics::CameraScaling,
        spitfire_input::{InputActionRef, InputConsume, InputMapping, VirtualAction},
        typid::ID,
        windowing::event::VirtualKeyCode,
    },
};
use std::collections::HashMap;

pub struct Gameplay {
    player: Character<PlayerState>,
    enemies: HashMap<ID<EnemyState>, Character<EnemyState>>,
    items: HashMap<ID<Item>, Item>,
    exit: InputActionRef,
    exit_handle: Option<ID<InputMapping>>,
    map_radius: f32,
}

impl Default for Gameplay {
    fn default() -> Self {
        Self {
            player: PlayerState::new_character(),
            enemies: Default::default(),
            items: Default::default(),
            exit: Default::default(),
            exit_handle: None,
            map_radius: 500.0,
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

        for _ in 0..4 {
            let position = [
                thread_rng().gen_range((-self.map_radius)..=self.map_radius),
                thread_rng().gen_range((-self.map_radius)..=self.map_radius),
                0.0,
            ];
            self.enemies.insert(
                ID::new(),
                EnemyState::new_character(position).activated(&mut context),
            );
        }

        for _ in 0..15 {
            let position = [
                thread_rng().gen_range((-self.map_radius)..=self.map_radius),
                thread_rng().gen_range((-self.map_radius)..=self.map_radius),
            ];
            self.items
                .insert(ID::new(), Item::new(ItemKind::random(), position));
        }
    }

    fn exit(&mut self, mut context: GameContext) {
        self.player.deactivate(&mut context);

        for (_, mut enemy) in self.enemies.drain() {
            enemy.deactivate(&mut context);
        }

        if let Some(id) = self.exit_handle {
            context.input.remove_mapping(id);
            self.exit_handle = None;
        }
    }

    fn update(&mut self, mut context: GameContext, delta_time: f32) {
        Events::maintain();

        if self.exit.get().is_down() {
            *context.state_change = GameStateChange::Pop;
        }

        self.player.update(&mut context, delta_time);

        for enemy in self.enemies.values_mut() {
            enemy.update(&mut context, delta_time);
            enemy
                .state
                .write()
                .unwrap()
                .sense_player(&self.player.state.read().unwrap());
        }

        // naive solution. use space partitioning for bigger scale worlds.
        for (id, item) in &self.items {
            let mut state = self.player.state.write().unwrap();
            if item.does_collide(state.sprite.transform.position.xy()) {
                state.consume_item(item);
                Events::write(Event::KillItem { id: *id });
            }

            for enemy in self.enemies.values_mut() {
                let mut state = enemy.state.write().unwrap();
                if item.does_collide(state.sprite.transform.position.xy()) {
                    state.consume_item(item);
                    Events::write(Event::KillItem { id: *id });
                }
            }
        }

        Events::read(|events| {
            self.player.state.write().unwrap().execute_events(events);

            for (id, enemy) in &mut self.enemies {
                enemy.state.write().unwrap().execute_events(*id, events);
            }

            for event in events {
                match event {
                    Event::KillPlayer => {
                        *context.state_change = GameStateChange::Pop;
                    }
                    Event::KillEnemy { id } => {
                        self.enemies.remove(id);
                    }
                    Event::KillItem { id } => {
                        self.items.remove(id);
                    }
                    _ => {}
                }
            }
        });
    }

    fn draw(&mut self, mut context: GameContext) {
        for item in self.items.values_mut() {
            item.draw(&mut context);
        }

        for enemy in self.enemies.values_mut() {
            enemy.draw(&mut context);
        }

        self.player.draw(&mut context);
    }

    fn draw_gui(&mut self, context: GameContext) {
        content_box((), || {
            {
                let state = self.player.state.read().unwrap();
                let layout = world_to_screen_content_layout(
                    state.sprite.transform.position.xy(),
                    Rect {
                        left: -50.0,
                        right: 50.0,
                        top: -60.0,
                        bottom: -45.0,
                    },
                    &context,
                );

                health_bar(layout, state.health);
            }

            for enemy in self.enemies.values() {
                let state = enemy.state.read().unwrap();
                let layout = world_to_screen_content_layout(
                    state.sprite.transform.position.xy(),
                    Rect {
                        left: -50.0,
                        right: 50.0,
                        top: -60.0,
                        bottom: -45.0,
                    },
                    &context,
                );

                health_bar(layout, state.health);
            }
        });
    }
}