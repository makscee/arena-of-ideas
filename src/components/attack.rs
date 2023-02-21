use super::*;

#[derive(Clone)]
pub struct AttackComponent {
    pub value: Hp,
    pub initial_value: Hp,
}

impl AttackComponent {
    pub fn new(value: Hp) -> Self {
        Self {
            value,
            initial_value: value,
        }
    }
}
