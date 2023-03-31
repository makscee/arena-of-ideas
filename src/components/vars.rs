use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarName {
    Damage,
    HpOriginalValue,
    HpValue,
    HpDamage,
    HpExtra,
    AttackOriginalValue,
    AttackValue,
    AttackExtra,
    Position,
    Radius,
    Size,
    Test,
    Faction,
    FactionColor,
    Slot,
    Card,
    Zoom,
    Description,
    HouseColor1,
    HouseColor2,
    HouseColor3,
    Floor,
    FieldPosition,
    Charges,
    Hits,
    Reflection,
    GrowAmount,
    Color,
    GlobalTime,
    StatusName,
    Store,
    G,
    BuyPrice,
    SellPrice,
    RerollPrice,
}

impl VarName {
    pub fn convert_to_uniform(&self) -> String {
        let mut name = "u".to_string();
        for c in self.to_string().chars() {
            if c.is_uppercase() {
                name.push('_');
                name.extend(c.to_lowercase());
            } else {
                name.push(c);
            }
        }

        name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Var {
    Int(i32),
    Float(f32),
    String((usize, String)),
    Vec2(vec2<f32>),
    Vec3(vec3<f32>),
    Vec4(vec4<f32>),
    Color(Rgba<f32>),
    Faction(Faction),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Vars(HashMap<VarName, Var>);

impl Vars {
    pub fn insert(&mut self, var: VarName, value: Var) {
        self.0.insert(var, value);
    }

    pub fn remove(&mut self, var: &VarName) {
        self.0.remove(var);
    }

    pub fn get(&self, var: &VarName) -> &Var {
        self.0
            .get(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get(&self, var: &VarName) -> Option<&Var> {
        self.0.get(var)
    }

    pub fn get_color(&self, var: &VarName) -> Rgba<f32> {
        self.try_get_color(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_color(&self, var: &VarName) -> Option<Rgba<f32>> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::Color(value) => Some(*value),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn get_vec2(&self, var: &VarName) -> vec2<f32> {
        self.try_get_vec2(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_vec2(&self, var: &VarName) -> Option<vec2<f32>> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::Vec2(value) => Some(*value),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn get_int(&self, var: &VarName) -> i32 {
        self.try_get_int(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_int(&self, var: &VarName) -> Option<i32> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::Int(value) => Some(*value),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn change_int(&mut self, var: &VarName, delta: i32) {
        let value = self.try_get_int(var).unwrap_or_default();
        self.insert(*var, Var::Int(value + delta));
    }

    pub fn set_int(&mut self, var: &VarName, value: i32) {
        self.insert(*var, Var::Int(value));
    }

    pub fn get_float(&self, var: &VarName) -> f32 {
        self.try_get_float(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_float(&self, var: &VarName) -> Option<f32> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::Float(value) => Some(*value),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn get_faction(&self, var: &VarName) -> Faction {
        self.try_get_faction(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_faction(&self, var: &VarName) -> Option<Faction> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::Faction(value) => Some(*value),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn get_string(&self, var: &VarName) -> String {
        self.try_get_string(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_string(&self, var: &VarName) -> Option<String> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::String((_font, value)) => Some(value.clone()),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn merge_mut(&mut self, other: &Vars, force: bool) {
        other.0.iter().for_each(|(key, value)| {
            if force || !self.0.contains_key(key) {
                self.0.insert(*key, value.clone());
            }
        });
    }
}

impl From<Vars> for ShaderUniforms {
    fn from(value: Vars) -> Self {
        let mut map: HashMap<String, ShaderUniform> = default();
        value.0.iter().for_each(|(name, value)| {
            let name = name.convert_to_uniform();
            match value {
                Var::Int(v) => {
                    map.insert(name, ShaderUniform::Int(*v));
                }
                Var::Float(v) => {
                    map.insert(name, ShaderUniform::Float(*v));
                }
                Var::String(text) => {
                    map.insert(name, ShaderUniform::String(text.clone()));
                }
                Var::Vec2(v) => {
                    map.insert(name, ShaderUniform::Vec2(*v));
                }
                Var::Vec3(v) => {
                    map.insert(name, ShaderUniform::Vec3(*v));
                }
                Var::Vec4(v) => {
                    map.insert(name, ShaderUniform::Vec4(*v));
                }
                Var::Color(v) => {
                    map.insert(name, ShaderUniform::Color(*v));
                }
                Var::Faction(_) => {}
            };
        });
        ShaderUniforms::from(map)
    }
}

impl fmt::Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait VarsProvider {
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources);
}
