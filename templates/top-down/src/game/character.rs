use super::game_object::GameObject;
use micro_games_kit::{
    context::GameContext,
    third_party::{
        emergent::task::Task,
        raui_core::{Managed, ManagedRefMut},
        spitfire_input::{InputMapping, InputMappingRef},
        typid::ID,
    },
};

pub struct CharacterMemory<State> {
    pub delta_time: f32,
    pub state: ManagedRefMut<State>,
}

#[derive(Default)]
pub enum CharacterController<State> {
    #[default]
    None,
    Input {
        mapping: InputMappingRef,
        id: Option<ID<InputMapping>>,
    },
    Ai(Box<dyn Task<CharacterMemory<State>>>),
}

impl<State> CharacterController<State> {
    pub fn input(mapping: impl Into<InputMappingRef>) -> Self {
        Self::Input {
            mapping: mapping.into(),
            id: None,
        }
    }

    pub fn ai(task: impl Task<CharacterMemory<State>> + 'static) -> Self {
        Self::Ai(Box::new(task))
    }
}

pub struct Character<State: GameObject> {
    pub state: Managed<State>,
    task: Box<dyn Task<CharacterMemory<State>>>,
    controller: CharacterController<State>,
}

impl<State: GameObject> Character<State> {
    pub fn new(
        state: State,
        task: impl Task<CharacterMemory<State>> + 'static,
        controller: CharacterController<State>,
    ) -> Self {
        Self {
            state: Managed::new(state),
            task: Box::new(task),
            controller,
        }
    }

    fn memory(&mut self, delta_time: f32) -> CharacterMemory<State> {
        CharacterMemory {
            delta_time,
            state: self.state.borrow_mut().unwrap(),
        }
    }
}

impl<State: GameObject> GameObject for Character<State> {
    fn activate(&mut self, context: &mut GameContext) {
        if let CharacterController::Input { mapping, id } = &mut self.controller {
            if id.is_none() {
                *id = Some(context.input.push_mapping(mapping.to_owned()));
            }
        }
        self.state.write().unwrap().activate(context);
        let mut memory = self.memory(0.0);
        self.task.on_enter(&mut memory);
    }

    fn deactivate(&mut self, context: &mut GameContext) {
        if let CharacterController::Input { id, .. } = &mut self.controller {
            if let Some(id) = id {
                context.input.remove_mapping(*id);
            }
            *id = None;
        }
        let mut memory = self.memory(0.0);
        self.task.on_exit(&mut memory);
        self.state.write().unwrap().deactivate(context);
    }

    fn update(&mut self, context: &mut GameContext, delta_time: f32) {
        let mut memory = self.memory(delta_time);
        self.task.on_process(&mut memory);
        self.task.on_update(&mut memory);
        self.state.write().unwrap().update(context, delta_time);
    }

    fn draw(&mut self, context: &mut GameContext) {
        self.state.write().unwrap().draw(context);
    }
}
