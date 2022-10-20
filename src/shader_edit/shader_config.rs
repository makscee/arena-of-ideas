use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
#[derive(Default)]
pub struct ClanShaderConfig {
    pub path: String,
    pub name: String,
    pub parameters: Vec<ClanShaderParam>,
}

#[derive(Serialize, Deserialize, Clone)]

pub struct ClanShaderParam {
    pub name: String,
    pub id: String,
    pub value: ClanShaderType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ClanShaderType {
    Enum {
        #[serde(default)]
        value: String,
        values: Vec<String>,
        show_all: bool,
    },
    Int {
        #[serde(default)]
        value: i64,
        range: Vec<i32>,
    },
    Float {
        #[serde(default)]
        value: f64,
        range: Vec<f32>,
    },
    Vector {
        #[serde(default = "zero_vec")]
        value: Vec2<f64>,
        range: Vec<f32>,
    },
}

fn zero_vec() -> Vec2<f64> {
    Vec2::ZERO
}

impl fmt::Display for ClanShaderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
