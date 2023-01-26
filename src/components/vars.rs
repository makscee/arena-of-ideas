use super::*;

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum VarName {
    Damage,
}

pub struct Vars {
    values_int: HashMap<VarName, i32>,
    values_entity: HashMap<VarName, legion::Entity>,
    values_string: HashMap<VarName, String>,
}

impl Vars {
    pub fn new() -> Self {
        Self {
            values_int: default(),
            values_entity: default(),
            values_string: default(),
        }
    }

    pub fn get_int(&self, name: &VarName) -> i32 {
        match self.values_int.get(name) {
            Some(value) => *value,
            None => {
                error!("Tried to get absent int value: {}", name);
                0
            }
        }
    }
}

impl fmt::Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
