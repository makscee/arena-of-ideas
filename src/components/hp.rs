use super::*;

pub type Hp = i32;
pub struct HpComponent {
    pub current: Hp,
    pub max: Hp,
}

impl HpComponent {
    pub fn new(context: &mut Context, max: Hp) -> Self {
        Self { current: max, max }
    }
}

impl VarsProvider for HpComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Hp_current, Var::Int(self.current));
        vars.insert(VarName::Hp_max, Var::Int(self.max));
    }
}
