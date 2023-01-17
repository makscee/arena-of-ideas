use super::*;

#[derive(Deserialize, Clone)]
pub struct UnitTemplate {
    pub name: String,
    pub tier: i32,
    pub description: String,
    pub health: i32,
    pub attack: i32,
    pub clans: Vec<Clan>,
    pub render: ShaderConfig,
    #[serde(default = "default_renders")]
    pub clan_renders: Vec<Vec<ShaderConfig>>,
    pub statuses: Vec<Status>,
    #[serde(default)]
    pub vars: HashMap<VarName, Expr>,
}

fn default_renders() -> Vec<Vec<ShaderConfig>> {
    let mut result: Vec<Vec<ShaderConfig>> = vec![];
    (0..MAX_LEVEL).into_iter().for_each(|level| {
        result.push(vec![]);
    });
    result
}
