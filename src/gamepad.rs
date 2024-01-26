use gilrs::{Axis, Button, GamepadId, Gilrs};
use spitfire_input::{InputActionOrAxisRef, InputAxis};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

thread_local! {
    static INSTANCE: Option<RefCell<Gilrs>> = Gilrs::new().ok().map(RefCell::new);
    static GAMEPADS: RefCell<HashSet<GamepadId>> = Default::default();
}

#[derive(Debug, Clone)]
pub enum GamepadInputAxis {
    Single {
        input: InputActionOrAxisRef,
        deadzone: f32,
    },
    Double {
        negative: InputActionOrAxisRef,
        positive: InputActionOrAxisRef,
        deadzone: f32,
    },
}

impl GamepadInputAxis {
    pub fn single(input: impl Into<InputActionOrAxisRef>, deadzone: f32) -> Self {
        Self::Single {
            input: input.into(),
            deadzone,
        }
    }

    pub fn double(
        negative: impl Into<InputActionOrAxisRef>,
        positive: impl Into<InputActionOrAxisRef>,
        deadzone: f32,
    ) -> Self {
        Self::Double {
            negative: negative.into(),
            positive: positive.into(),
            deadzone,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GamepadInput {
    id: Option<GamepadId>,
    pub buttons: HashMap<Button, InputActionOrAxisRef>,
    pub axes: HashMap<Axis, GamepadInputAxis>,
    pub auto_acquire: bool,
}

impl Drop for GamepadInput {
    fn drop(&mut self) {
        self.release();
    }
}

impl GamepadInput {
    pub fn is_supported() -> bool {
        INSTANCE
            .try_with(|instance| instance.is_some())
            .unwrap_or_default()
    }

    pub fn new() -> Option<Self> {
        let mut result = Self::default();
        if result.acquire() {
            Some(result)
        } else {
            None
        }
    }

    pub fn acquire(&mut self) -> bool {
        self.release();
        INSTANCE
            .try_with(|instance| {
                if let Some(instance) = instance {
                    let instance = instance.borrow();
                    return GAMEPADS
                        .try_with(|gamepads| {
                            let mut gamepads = gamepads.borrow_mut();
                            for (id, _) in instance.gamepads() {
                                if !gamepads.contains(&id) {
                                    gamepads.insert(id);
                                    self.id = Some(id);
                                    return true;
                                }
                            }
                            false
                        })
                        .unwrap_or_default();
                }
                false
            })
            .unwrap_or_default()
    }

    pub fn release(&mut self) {
        let _ = GAMEPADS.try_with(|gamepads| {
            if let Some(id) = self.id {
                gamepads.borrow_mut().remove(&id);
            }
        });
        self.id = None;
    }

    pub fn id(&self) -> Option<GamepadId> {
        self.id
    }

    pub fn is_connected(&self) -> bool {
        self.id.is_some()
    }

    pub fn button(mut self, id: Button, input: impl Into<InputActionOrAxisRef>) -> Self {
        self.buttons.insert(id, input.into());
        self
    }

    pub fn axis(mut self, id: Axis, input: impl Into<GamepadInputAxis>) -> Self {
        self.axes.insert(id, input.into());
        self
    }

    pub fn auto_acquire(mut self) -> Self {
        self.auto_acquire = true;
        self
    }

    pub fn maintain() {
        let _ = INSTANCE.try_with(|instance| {
            if let Some(instance) = instance {
                let mut instance = instance.borrow_mut();
                while instance.next_event().is_some() {}
            }
        });
    }

    pub fn apply(&mut self) {
        if self.auto_acquire && !self.is_connected() {
            self.acquire();
        }
        if let Some(id) = self.id {
            let _ = INSTANCE.try_with(|instance| {
                if let Some(instance) = instance {
                    let instance = instance.borrow();
                    let _ = GAMEPADS.try_with(|gamepads| {
                        if let Some(gamepad) = instance.connected_gamepad(id) {
                            for (id, input) in &mut self.buttons {
                                if let Some(data) = gamepad.button_data(*id) {
                                    match input {
                                        InputActionOrAxisRef::Action(input) => {
                                            input.set(input.get().change(data.is_pressed()));
                                        }
                                        InputActionOrAxisRef::Axis(input) => {
                                            input.set(InputAxis(data.value()));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            for (id, input) in &mut self.axes {
                                if let Some(data) = gamepad.axis_data(*id) {
                                    match input {
                                        GamepadInputAxis::Single { input, deadzone } => {
                                            let mut value = data.value().abs();
                                            if value < *deadzone {
                                                value = 0.0;
                                            }
                                            match input {
                                                InputActionOrAxisRef::Action(input) => {
                                                    input.set(input.get().change(value > 0.5));
                                                }
                                                InputActionOrAxisRef::Axis(input) => {
                                                    input.set(InputAxis(value));
                                                }
                                                _ => {}
                                            }
                                        }
                                        GamepadInputAxis::Double {
                                            negative,
                                            positive,
                                            deadzone,
                                        } => {
                                            let mut value = data.value();
                                            if value.abs() < *deadzone {
                                                value = 0.0;
                                            }
                                            match positive {
                                                InputActionOrAxisRef::Action(input) => {
                                                    input.set(input.get().change(value > 0.5));
                                                }
                                                InputActionOrAxisRef::Axis(input) => {
                                                    input.set(InputAxis(value.max(0.0)));
                                                }
                                                _ => {}
                                            }
                                            match negative {
                                                InputActionOrAxisRef::Action(input) => {
                                                    input.set(input.get().change(value < -0.5));
                                                }
                                                InputActionOrAxisRef::Axis(input) => {
                                                    input.set(InputAxis(-value.min(0.0)));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            gamepads.borrow_mut().remove(&id);
                            self.id = None;
                        }
                    });
                }
            });
        }
    }
}
