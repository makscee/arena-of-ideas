use core::any::Any;

use super::*;

pub type EventListener = Box<dyn Fn(&mut Logic) + 'static>;

pub struct Events {
    mappings: HashMap<String, GameEvent>,
    listeners: HashMap<GameEvent, Vec<EventListener>>,
}

impl Events {
    pub fn new(mappings: Vec<KeyMapping>) -> Self {
        let mappings: HashMap<String, GameEvent> = mappings
            .into_iter()
            .map(|key_mapping| (key_mapping.key, key_mapping.event))
            .collect();
        Self {
            listeners: hashmap! {},
            mappings,
        }
    }

    pub fn add_listener(&mut self, event: GameEvent, listener: EventListener) {
        self.listeners
            .entry(event)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn trigger(&self, event: GameEvent, logic: &mut Logic) {
        if self.listeners.contains_key(&event) {
            for listener in &self.listeners[&event] {
                listener(logic);
            }
        }
    }

    pub fn trigger_by_key(&mut self, key: String, logic: &mut Logic) {
        if self.mappings.contains_key(&key) {
            self.trigger(self.mappings[&key], logic);
        }
    }
}

#[derive(Eq, Hash, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum GameEvent {
    Ability,
    Pause,
    TickBack,
    TickForw,
}
