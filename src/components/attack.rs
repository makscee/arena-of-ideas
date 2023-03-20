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
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources) {
        vars.insert(VarName::AttackValue, Var::Int(self.value as i32))
    }
}
