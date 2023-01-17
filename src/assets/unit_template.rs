use super::*;

#[derive(Deserialize, Clone)]
pub struct UnitTemplate {
    pub name: String,
    pub tier: i32,
    pub description: String,
    pub health: i32,
    pub attack: i32,
    pub clans: Vec<Clan>,
    #[serde(default = "default_renders")]
    pub layers: Vec<ShaderProgram>,
    #[serde(default)]
    pub vars: HashMap<VarName, Expr>,
}

fn default_renders() -> Vec<ShaderProgram> {
    let mut result: Vec<ShaderProgram> = vec![];
    result
}
