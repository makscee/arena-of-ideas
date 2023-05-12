use std::str::FromStr;

use geng::prelude::itertools::Itertools;
use strum_macros::Display;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum VarName {
    Damage,
    HpOriginalValue,
    HpValue,
    HpStr,
    HpDamage,
    HpExtra,
    AttackOriginalValue,
    AttackValue,
    AttackStr,
    AttackExtra,
    Position,
    Radius,
    Box,
    Size,
    Scale,
    Test,
    Faction,
    FactionColor,
    Slot,
    Slots,
    Card,
    Zoom,
    Description,
    House,
    HouseColor,
    Level,
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
    FreeRerolls,
    Persistent,
    Rank,
    Rank1,
    Rank2,
    Rank3,
    BackgroundLight,
    BackgroundDark,
    OutlineColor,
    LastAttacker,
    LastHealer,
}

impl VarName {
    pub fn uniform(&self) -> String {
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
    Entity(legion::Entity),
}

impl Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Var::Int(v) => write!(f, "{v}"),
            Var::Float(v) => write!(f, "{v}"),
            Var::String(v) => write!(f, "{} ({})", v.1, v.0),
            Var::Vec2(v) => write!(f, "{v}"),
            Var::Vec3(v) => write!(f, "{v}"),
            Var::Vec4(v) => write!(f, "{v}"),
            Var::Color(v) => write!(f, "{v}"),
            Var::Faction(v) => write!(f, "{v}"),
            Var::Entity(v) => write!(f, "{v:?}"),
        }
    }
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

    pub fn set_color(&mut self, var: &VarName, value: Rgba<f32>) {
        self.insert(*var, Var::Color(value));
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

    pub fn change_vec2(&mut self, var: &VarName, delta: vec2<f32>) {
        let value = self.try_get_vec2(var).unwrap_or(vec2::ZERO);
        self.insert(*var, Var::Vec2(value + delta));
    }

    pub fn set_vec2(&mut self, var: &VarName, value: vec2<f32>) {
        self.insert(*var, Var::Vec2(value));
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

    pub fn change_float(&mut self, var: &VarName, delta: f32) {
        let value = self.try_get_float(var).unwrap_or_default();
        self.insert(*var, Var::Float(value + delta));
    }

    pub fn set_float(&mut self, var: &VarName, value: f32) {
        self.insert(*var, Var::Float(value));
    }

    pub fn set_faction(&mut self, var: &VarName, value: Faction) {
        self.insert(*var, Var::Faction(value));
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

    pub fn set_entity(&mut self, var: &VarName, value: legion::Entity) {
        self.insert(*var, Var::Entity(value));
    }

    pub fn get_entity(&self, var: &VarName) -> legion::Entity {
        self.try_get_entity(var)
            .expect(&format!("Failed to get var {}", var))
    }

    pub fn try_get_entity(&self, var: &VarName) -> Option<legion::Entity> {
        match self.try_get(var) {
            Some(value) => match value {
                Var::Entity(value) => Some(*value),
                _ => panic!("Wrong Var type {}", var),
            },
            None => None,
        }
    }

    pub fn set_string(&mut self, var: &VarName, font: usize, value: String) {
        self.insert(*var, Var::String((font, value)));
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

    pub fn try_get_house(&self) -> Option<HouseName> {
        self.try_get_string(&VarName::House)
            .and_then(|x| HouseName::from_str(x.as_str()).ok())
    }

    pub fn merge_mut(&mut self, other: &Vars, force: bool) {
        other.0.iter().for_each(|(key, value)| {
            if force || !self.0.contains_key(key) {
                self.0.insert(*key, value.clone());
            }
        });
    }
}

impl From<HashMap<VarName, Var>> for Vars {
    fn from(value: HashMap<VarName, Var>) -> Self {
        Self(value)
    }
}

impl Display for Vars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self
            .0
            .iter()
            .map(|(key, value)| format!("{key} -> {value}"))
            .join(" ");
        write!(f, "{text}")
    }
}

impl From<Vars> for ShaderUniforms {
    fn from(value: Vars) -> Self {
        let mut map: HashMap<String, ShaderUniform> = default();
        value.0.iter().for_each(|(name, value)| {
            let name = name.uniform();
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
                Var::Entity(_) | Var::Faction(_) => {}
            };
        });
        ShaderUniforms::from(map)
    }
}

pub trait ContextExtender {
    fn extend(&self, context: &mut Context, resources: &Resources);
}
