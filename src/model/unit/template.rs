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
        }
    }
}

impl geng::LoadAsset for UnitTemplate {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut result: Self = serde_json::from_str(&json)?;
            result.long_name = file_name;
            result.path = path.to_str().unwrap().to_string();
            Ok(result)
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

#[derive(Deref, DerefMut, Clone)]
pub struct UnitTemplates {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, UnitTemplate>,
}
