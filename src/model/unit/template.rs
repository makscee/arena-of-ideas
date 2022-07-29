use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    pub name: UnitType,
    pub long_name: UnitType,
    /// Units with tier equal to 0 are not included in the shop
    pub tier: Tier,
    /// Description displayed on the unit card
    pub description: String,
    /// Units `triple` set to Some get converted to that unit once three of them are bought by the player
    #[serde(default)]
    pub triple: Option<UnitType>,
    pub health: Health,
    pub base_damage: Health,
    pub block: R32,
    pub crit_chance: R32,
    pub action_speed: R32,
    pub spawn_animation_time: Time,
    pub radius: R32,
    pub action: ActionProperties,
    pub cooldown: Ticks,
    pub range: Coord,
    pub statuses: Vec<StatusRef>,
    pub ability: Option<Ability>,
    pub clans: Vec<Clan>,
    #[serde(rename = "render")]
    pub render_config: ShaderConfig,
    pub base: Option<UnitType>,
}

impl Default for UnitTemplate {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            long_name: "".to_string(),
            tier: 0,
            description: String::new(),
            triple: None,
            health: Health::new(1.0),
            base_damage: Health::new(1.0),
            block: r32(0.0),
            crit_chance: r32(0.0),
            action_speed: r32(1.0),
            spawn_animation_time: Time::new(0.0),
            radius: R32::new(0.5),
            action: ActionProperties { effect: default() },
            range: 1,
            cooldown: 1,
            statuses: default(),
            ability: None,
            render_config: ShaderConfig {
                path: "".to_string(),
                instances: 1,
                vertices: 1,
                parameters: default(),
            },
            clans: default(),
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
