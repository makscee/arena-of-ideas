use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
#[derive(Default)]
pub struct ClanShaderConfig {
    pub path: String,
    pub name: String,
    pub parameters: HashMap<String, ClanShaderParam>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
#[derive(Default)]
pub struct ClanShaderParam {
    pub name: String,
    pub id: String,
    pub r#type: ShaderWidgetType,
    pub range: Option<Vec<f32>>,
    pub values: Option<Vec<String>>,
    pub show_all: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum ShaderWidgetType {
    #[default]
    Enum,
    Int,
    Float,
    Vector,
}

impl fmt::Display for ShaderWidgetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
