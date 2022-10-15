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
    name: String,
    r#type: String,
    range: Option<Vec<f32>>,
    values: Option<Vec<String>>,
    show_all: Option<bool>,
}
