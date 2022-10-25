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
        values: Vec<String>,
        #[serde(default)]
        show_all: bool,
    },
    Int {
        range: Vec<i32>,
        default: Option<i32>,
    },
    Float {
        range: Vec<f32>,
        default: Option<f32>,
    },
    Vector {
        range: Vec<f32>,
        default: Option<Vec2<f32>>,
    },
}

impl fmt::Display for ClanShaderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
