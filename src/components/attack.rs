use super::*;

#[derive(Clone)]
pub struct AttackComponent {
    pub value: usize,
}

impl AttackComponent {
    pub fn new(value: usize) -> Self {
        Self { value }
    }
}

impl VarsProvider for AttackComponent {
    fn extend_vars(&self, vars: &mut Vars, _: &Resources) {
        vars.insert(VarName::AttackValue, Var::Int(self.value as i32));
        vars.insert(VarName::AttackOriginalValue, Var::Int(self.value as i32));
    }
}
