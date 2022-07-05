use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    /// Units with tier equal to 0 are not included in the shop
    pub tier: Tier,
    /// Description displayed on the unit card
    pub description: String,
    /// Units `triple` set to Some get converted to that unit once three of them are bought by the player
    #[serde(default)]
    pub triple: Option<UnitType>,
    pub health: Health,
    pub base_damage: Health,
    pub armor: R32,
    pub armor_penetration: R32,
    pub crit_chance: R32,
    pub action_speed: R32,
    pub spawn_animation_time: Time,
    pub radius: R32,
    pub action: ActionProperties,
    pub statuses: Vec<StatusRef>,
    pub target_ai: TargetAi,
    pub ability: Option<Ability>,
    pub clans: Vec<Clan>,
    #[serde(rename = "render")]
    pub render_config: RenderConfig,
}

impl Default for UnitTemplate {
    fn default() -> Self {
        Self {
            tier: 0,
            description: String::new(),
            triple: None,
            health: Health::new(1.0),
            base_damage: Health::new(1.0),
            armor: r32(0.0),
            armor_penetration: r32(0.0),
            crit_chance: r32(0.0),
            action_speed: r32(1.0),
            spawn_animation_time: Time::new(0.0),
            radius: R32::new(0.5),
            action: ActionProperties {
                range: 1,
                cooldown: Time::new(1.0),
                animation_delay: Time::new(1.0),
                effect: default(),
            },
            statuses: default(),
            target_ai: TargetAi::Closest,
            ability: None,
            render_config: RenderConfig::Circle {
                color: Color::WHITE,
            },
            clans: default(),
        }
    }
}

impl geng::LoadAsset for UnitTemplate {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut result: Self = serde_json::from_str(&json)?;
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
