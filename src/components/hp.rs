use super::*;

pub type Hp = i32;
#[derive(Clone)]
pub struct HpComponent {
    pub current: Hp,
    pub max: Hp,
}

impl HpComponent {
    pub fn new(max: Hp) -> Self {
        Self { current: max, max }
    }
}

impl VarsProvider for HpComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::HpCurrent, Var::Int(self.current));
        vars.insert(VarName::HpMax, Var::Int(self.max));
    }
}
