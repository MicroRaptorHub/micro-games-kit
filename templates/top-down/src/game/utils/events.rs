use crate::game::enemy::EnemyState;
use micro_games_kit::third_party::{typid::ID, vek::Vec2};
use std::cell::RefCell;

thread_local! {
    static PENDING: RefCell<Vec<Event>> = Default::default();
    static RECEIVED: RefCell<Vec<Event>> = Default::default();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instigator {
    Player,
    Enemy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    KillPlayer,
    KillEnemy {
        id: ID<EnemyState>,
    },
    Attack {
        position: Vec2<f32>,
        range: f32,
        value: usize,
        instigator: Instigator,
    },
}

#[derive(Default)]
pub struct Events;

impl Events {
    pub fn write(event: Event) {
        PENDING.with(|pending| pending.borrow_mut().push(event));
    }

    pub fn read<R>(mut f: impl FnMut(&[Event]) -> R) -> R {
        RECEIVED.with(|received| f(&received.borrow()))
    }

    pub fn maintain() {
        PENDING.with(|pending| {
            RECEIVED.with(|received| {
                let pending: &mut Vec<_> = &mut pending.borrow_mut();
                let received: &mut Vec<_> = &mut received.borrow_mut();
                std::mem::swap(pending, received);
                pending.clear();
            });
        });
    }
}
