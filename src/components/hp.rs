use super::*;

pub type Hp = i32;
pub struct HpComponent {
    pub current: Hp,
    pub max: Hp,
}

impl HpComponent {
    pub fn new(max: Hp) -> Self {
        Self { current: max, max }
    }
}
