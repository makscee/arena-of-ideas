use super::*;

pub type Hp = i32;
#[derive(Clone)]
pub struct HpComponent {
    current: Hp,
    pub max: Hp,

    pub last_change: Time,
}

impl HpComponent {
    pub fn new(max: Hp) -> Self {
        Self {
            current: max,
            max,
            last_change: -100.0,
        }
    }

    pub fn current(&self) -> Hp {
        self.current
    }

    pub fn set_current(&mut self, current: Hp, resources: &Resources) {
        if current < self.current {
            self.last_change = resources.cassette.last_start();
        }
        self.current = current;
    }
}

impl VarsProvider for HpComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::HpCurrent, Var::Int(self.current));
        vars.insert(VarName::HpMax, Var::Int(self.max));
        vars.insert(VarName::HpLastDmg, Var::Float(self.last_change));
    }
}
