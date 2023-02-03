use super::*;

pub type Hp = i32;
pub struct HpComponent {
    current: Hp,
    pub max: Hp,

    last_dmg: Time,
}

impl HpComponent {
    pub fn new(max: Hp) -> Self {
        Self {
            current: max,
            max,
            last_dmg: -100.0,
        }
    }

    pub fn current(&self) -> Hp {
        self.current
    }

    pub fn set_current(&mut self, current: Hp, resources: &Resources) {
        if current < self.current {
            self.last_dmg = resources.cassette.last_start();
        }
        self.current = current;
    }
}

impl VarsProvider for HpComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::HpCurrent, Var::Int(self.current));
        vars.insert(VarName::HpMax, Var::Int(self.max));
        vars.insert(VarName::HpLastDmg, Var::Float(self.last_dmg));
    }
}
