use super::*;

pub type Hp = i32;
pub struct HpComponent {
    pub current: VarInt,
    pub max: VarInt,
}

impl HpComponent {
    pub fn new(context: &mut Context, max: Hp) -> Self {
        let current_var = VarInt::new(VarName::Hp_current);
        let max_var = VarInt::new(VarName::Hp_max);
        max_var.set(&mut context.vars, max);
        current_var.set(&mut context.vars, max);
        Self {
            current: current_var,
            max: max_var,
        }
    }
}
