use super::*;

#[derive(Clone)]
pub struct AttackComponent {
    value: Hp,
    initial_value: Hp,
    last_change: Time,
}

impl AttackComponent {
    pub fn new(value: Hp) -> Self {
        Self {
            value,
            initial_value: value,
            last_change: -100.0,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn initial_value(&self) -> i32 {
        self.initial_value
    }

    pub fn last_change(&self) -> Time {
        self.last_change
    }

    pub fn set_value(&mut self, value: Hp, resources: &Resources) {
        if value != self.value {
            self.last_change = resources.cassette.last_start();
        }
        self.value = value;
    }
}
