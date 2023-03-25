use super::*;

pub type Hp = i32;
#[derive(Clone)]
pub struct HealthComponent {
    pub value: Hp,
    pub damage: usize,
}

impl HealthComponent {
    pub fn new(max: Hp, damage: usize) -> Self {
        Self { value: max, damage }
    }
}

impl VarsProvider for HealthComponent {
    fn extend_vars(&self, vars: &mut Vars, _: &Resources) {
        vars.insert(VarName::HpOriginalValue, Var::Int(self.value));
        vars.insert(VarName::HpValue, Var::Int(self.value));
        vars.insert(VarName::HpDamage, Var::Int(self.damage as i32));
    }
}
