use micro_games_kit::third_party::{
    kira::{
        manager::{AudioManager, AudioManagerSettings},
        sound::static_sound::{StaticSoundData, StaticSoundHandle},
    },
    raui_core::{Managed, ManagedRef, ManagedRefMut},
};
use std::{cell::RefCell, collections::HashMap, io::Cursor};

thread_local! {
    static INSTANCE: RefCell<Managed<Audio>> = Default::default();
}

pub struct Audio {
    pub manager: AudioManager,
    sounds: HashMap<String, StaticSoundData>,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            sounds: Default::default(),
        }
    }
}

impl Audio {
    pub fn read() -> ManagedRef<Self> {
        INSTANCE.with(|instance| instance.borrow().borrow().unwrap())
    }

    pub fn write() -> ManagedRefMut<Self> {
        INSTANCE.with(|instance| instance.borrow_mut().borrow_mut().unwrap())
    }

    pub fn sound(&self, id: &str) -> Option<StaticSoundData> {
        self.sounds.get(id).cloned()
    }

    pub fn register(&mut self, id: impl ToString, data: &'static [u8]) {
        self.sounds.insert(
            id.to_string(),
            StaticSoundData::from_cursor(Cursor::new(data)).unwrap(),
        );
    }

    pub fn play(&mut self, id: &str) -> Option<StaticSoundHandle> {
        self.manager.play(self.sounds.get(id)?.clone()).ok()
    }
}
