use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    pub name: UnitType,
    pub long_name: UnitType,
    pub path: String,
    /// Units with tier equal to 0 are not included in the shop
    pub tier: Tier,
    /// Description displayed on the unit card
    pub description: String,
    pub health: i32,
    pub attack: i32,
    #[serde(default = "default_stacks")]
    pub stacks: i32,
    pub spawn_animation_time: Time,
    pub radius: R32,
    #[serde(default)]
    pub action: Effect,
    pub statuses: Vec<StatusRef>,
    pub clans: Vec<Clan>,
    #[serde(rename = "render")]
    pub render_config: ShaderConfig,
    #[serde(default = "default_renders")]
    pub clan_renders: Vec<Vec<ShaderConfig>>, // level_index -> clan_index
    pub base: Option<UnitType>,
    #[serde(default)]
    pub vars: HashMap<VarName, Expr>,
}

fn default_stacks() -> i32 {
    1
}

fn default_renders() -> Vec<Vec<ShaderConfig>> {
    let mut result: Vec<Vec<ShaderConfig>> = vec![];
    (0..MAX_LEVEL).into_iter().for_each(|level| {
        result.push(vec![]);
    });
    result
}

impl Default for UnitTemplate {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            long_name: "".to_string(),
            path: "".to_string(),
            tier: 0,
            description: String::new(),
            health: 1,
            attack: 1,
            stacks: 1,
            spawn_animation_time: Time::new(0.0),
            radius: R32::new(0.5),
            action: default(),
            statuses: default(),
            render_config: ShaderConfig {
                path: "".to_string(),
                instances: 1,
                vertices: 1,
                parameters: default(),
            },
            clans: default(),
            clan_renders: default(),
            base: None,
            vars: hashmap! {},
        }
    }
}

#[derive(Deref, DerefMut, Clone)]
pub struct UnitTemplates {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, UnitTemplate>,
}
