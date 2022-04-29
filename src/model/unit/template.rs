use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    pub tier: Tier,
    pub health: Health,
    pub spawn_animation_time: Time,
    pub speed: Coord,
    pub radius: Coord,
    pub action: ActionProperties,
    pub move_ai: MoveAi,
    pub statuses: Vec<Status>,
    pub target_ai: TargetAi,
    pub ability: Option<Ability>,
    pub alliances: HashSet<Alliance>,
    #[serde(rename = "render")]
    pub render_config: RenderConfig,
}

impl UnitTemplate {
    pub fn walk_effects_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.action.effect.walk_mut(f);
        for status in &mut self.statuses {
            status.walk_effects_mut(f);
        }
        for ability in &mut self.ability {
            ability.effect.walk_mut(f);
        }
    }
}

impl Default for UnitTemplate {
    fn default() -> Self {
        Self {
            tier: Tier::new(1).unwrap(),
            health: Health::new(1.0),
            spawn_animation_time: Time::new(0.0),
            speed: Coord::new(1.0),
            radius: Coord::new(0.5),
            action: ActionProperties {
                range: Coord::new(1.0),
                cooldown: Time::new(1.0),
                animation_delay: Time::new(1.0),
                effect: default(),
            },
            statuses: default(),
            move_ai: MoveAi::Advance,
            target_ai: TargetAi::Closest,
            ability: None,
            render_config: RenderConfig::Circle {
                color: Color::WHITE,
            },
            alliances: default(),
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
