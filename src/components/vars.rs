use super::*;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarName {
    Damage,
    Hp_max,
    Hp_current,
    Test,
}

pub struct VarInt(VarName);

impl VarInt {
    pub fn new(name: VarName) -> Self {
        Self(name)
    }
    pub fn get(&self, vars: &Vars) -> Result<i32, Error> {
        vars.get_int(&self.0)
    }
    pub fn set(&self, vars: &mut Vars, value: i32) {
        vars.set_int(self.0.clone(), value);
    }
    pub fn change(&self, vars: &mut Vars, delta: i32) -> Result<i32, Error> {
        let result = vars.get_int(&self.0)? + delta;
        vars.set_int(self.0.clone(), result);
        Ok(result)
    }
}

pub struct VarEntity(VarName);

impl VarEntity {
    pub fn get(&self, vars: &Vars) -> Result<legion::Entity, Error> {
        vars.get_entity(&self.0)
    }
}

pub struct VarString(VarName);

impl VarString {
    pub fn get(&self, vars: &Vars) -> Result<String, Error> {
        vars.get_string(&self.0)
    }
}

#[derive(Clone, Debug, Default)]
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

    pub fn set_int(&mut self, name: VarName, value: i32) {
        self.values_int.insert(name, value);
    }

    pub fn get_int(&self, name: &VarName) -> Result<i32, Error> {
        self.values_int
            .get(name)
            .context(format!("Failed to get Int var {}", name))
            .cloned()
    }

    pub fn get_entity(&self, name: &VarName) -> Result<legion::Entity, Error> {
        self.values_entity
            .get(name)
            .context(format!("Failed to get Entity var {}", name))
            .cloned()
    }

    pub fn get_string(&self, name: &VarName) -> Result<String, Error> {
        self.values_string
            .get(name)
            .context(format!("Failed to get String var {}", name))
            .cloned()
    }

    /// Overrides that intersect, doesn't add new
    pub fn update(&self, other: &Vars) -> Vars {
        let mut context = self.clone();
        other.values_int.iter().for_each(|(key, value)| {
            if context.values_int.contains_key(key) {
                context.values_int.insert(key.clone(), value.clone());
            }
        });
        context
    }
    /// Overrides that intersect, doesn't add new
    pub fn update_self(&mut self, other: &Vars) {
        other.values_int.iter().for_each(|(key, value)| {
            if self.values_int.contains_key(key) {
                self.values_int.insert(key.clone(), value.clone());
            }
        });
    }

    pub fn override_self(&mut self, other: &Vars) {
        other.values_int.iter().for_each(|(key, value)| {
            self.values_int.insert(key.clone(), value.clone());
        });
    }

    pub fn extend(&self, other: &Vars) -> Vars {
        let mut context = self.clone();
        other.values_int.iter().for_each(|(key, value)| {
            if !context.values_int.contains_key(key) {
                context.values_int.insert(key.clone(), value.clone());
            }
        });
        context
    }

    pub fn extend_self(&mut self, other: &Vars) {
        other.values_int.iter().for_each(|(key, value)| {
            if !self.values_int.contains_key(key) {
                self.values_int.insert(key.clone(), value.clone());
            }
        });
    }
}

impl fmt::Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
