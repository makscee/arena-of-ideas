use super::*;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarName {
    Damage,
    Hp_max,
    Hp_current,
    Position,
    Test,
    Faction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Var {
    Int(i32),
    Float(f32),
    String(String),
    Vec2(Vec2<f32>),
    Vec3(Vec3<f32>),
    Vec4(Vec4<f32>),
    Color(Rgba<f32>),
}

#[derive(Clone, Debug, Default)]
pub struct Vars(HashMap<VarName, Var>);

impl Vars {
    pub fn insert(&mut self, name: VarName, var: Var) {
        self.0.insert(name, var);
    }
}

impl From<Vars> for ShaderUniforms {
    fn from(value: Vars) -> Self {
        let mut map: HashMap<String, ShaderUniform> = default();
        value.0.iter().for_each(|(name, value)| {
            let name = "u_".to_owned() + &name.to_string().to_lowercase();
            match value {
                Var::Int(v) => {
                    map.insert(name, ShaderUniform::Int(*v));
                }
                Var::Float(v) => {
                    map.insert(name, ShaderUniform::Float(*v));
                }
                Var::String(_) => {}
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
            };
        });
        ShaderUniforms::from(map)
    }
}

impl fmt::Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub trait VarsProvider {
    fn extend_vars(&self, vars: &mut Vars);
}
