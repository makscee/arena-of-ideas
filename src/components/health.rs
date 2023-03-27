use super::*;

pub type Hp = i32;
#[derive(Clone)]
pub struct HealthComponent {
    pub value: Hp,
    damage: usize,
    last_attacker: Option<legion::Entity>,
}

impl HealthComponent {
    pub fn new(max: Hp, damage: usize) -> Self {
        Self {
            value: max,
            damage,
            last_attacker: default(),
        }
    }

    pub fn deal_damage(&mut self, amount: usize, attacker: legion::Entity) {
        if amount == 0 {
            return;
        }
        self.last_attacker = Some(attacker);
        self.damage += amount;
    }

    pub fn heal_damage(&mut self, amount: usize) {
        if amount == 0 {
            return;
        }
        self.damage -= amount.max(self.damage);
    }

    pub fn last_attacker(&self) -> Option<legion::Entity> {
        self.last_attacker
    }

    pub fn stats(&self) -> (Hp, usize) {
        (self.value, self.damage)
    }
}

impl VarsProvider for HealthComponent {
    fn extend_vars(&self, vars: &mut Vars, _: &Resources) {
        vars.insert(VarName::HpOriginalValue, Var::Int(self.value));
        vars.insert(VarName::HpValue, Var::Int(self.value));
        vars.insert(VarName::HpDamage, Var::Int(self.damage as i32));
    }
}
