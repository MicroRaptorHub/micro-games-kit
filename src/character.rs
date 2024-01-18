use crate::{context::GameContext, game::GameObject};
use emergent::task::Task;
use raui_core::{Managed, ManagedRefMut};
use spitfire_input::{InputMapping, InputMappingRef};
use typid::ID;

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

    pub fn activated(mut self, context: &mut GameContext) -> Self {
        self.activate(context);
        self
    }
}

impl<State: GameObject> GameObject for Character<State> {
    fn activate(&mut self, context: &mut GameContext) {
        match &mut self.controller {
            CharacterController::None => {}
            CharacterController::Input { mapping, id } => {
                if id.is_none() {
                    *id = Some(context.input.push_mapping(mapping.to_owned()));
                }
            }
            CharacterController::Ai(task) => {
                task.on_enter(&mut CharacterMemory {
                    delta_time: 0.0,
                    state: self.state.borrow_mut().unwrap(),
                });
            }
        }
        let mut memory = CharacterMemory {
            delta_time: 0.0,
            state: self.state.borrow_mut().unwrap(),
        };
        self.state.write().unwrap().activate(context);
        self.task.on_enter(&mut memory);
    }

    fn deactivate(&mut self, context: &mut GameContext) {
        match &mut self.controller {
            CharacterController::None => {}
            CharacterController::Input { id, .. } => {
                if let Some(id) = id {
                    context.input.remove_mapping(*id);
                }
                *id = None;
            }
            CharacterController::Ai(task) => {
                task.on_exit(&mut CharacterMemory {
                    delta_time: 0.0,
                    state: self.state.borrow_mut().unwrap(),
                });
            }
        }
        let mut memory = CharacterMemory {
            delta_time: 0.0,
            state: self.state.borrow_mut().unwrap(),
        };
        self.task.on_exit(&mut memory);
        self.state.write().unwrap().deactivate(context);
    }

    fn process(&mut self, context: &mut GameContext, delta_time: f32) {
        if let CharacterController::Ai(task) = &mut self.controller {
            let mut memory = CharacterMemory {
                delta_time,
                state: self.state.borrow_mut().unwrap(),
            };
            task.on_process(&mut memory);
            task.on_update(&mut memory);
        }
        let mut memory = CharacterMemory {
            delta_time,
            state: self.state.borrow_mut().unwrap(),
        };
        self.task.on_process(&mut memory);
        self.task.on_update(&mut memory);
        self.state.write().unwrap().process(context, delta_time);
    }

    fn draw(&mut self, context: &mut GameContext) {
        self.state.write().unwrap().draw(context);
    }
}
