use super::*;

pub struct AttackComponent {
    pub value: Hp,
}

impl AttackComponent {
    pub fn new(value: Hp) -> Self {
        Self { value }
    }
}
