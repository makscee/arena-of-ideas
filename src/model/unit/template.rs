use super::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RenderConfig {
    Circle { color: Color<f32> },
    Texture { path: String },
    Shader { path: String },
}

#[derive(Clone)]
pub enum RenderMode {
    Circle { color: Color<f32> },
    Texture { texture: Rc<ugli::Texture> },
    Shader { program: Rc<ugli::Program> },
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Circle {
            color: Color::BLACK,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    pub hp: Health,
    pub spawn_animation_time: Time,
    pub speed: Coord,
    pub size: Coord,
    pub attack: AttackProperties,
    pub triggers: Vec<UnitTrigger>,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub abilities: HashMap<Key, Ability>,
    pub alliances: HashSet<Alliance>,
    #[serde(rename = "render")]
    pub render_config: RenderConfig,
    #[serde(skip)]
    pub render_mode: RenderMode,
}

impl UnitTemplate {
    pub fn walk_effects_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.attack.effect.walk_mut(f);
        for trigger in &mut self.triggers {
            match trigger {
                UnitTrigger::Death(effect) => effect.walk_mut(f),
                UnitTrigger::Spawn(effect) => effect.walk_mut(f),
                UnitTrigger::Kill(trigger) => {
                    trigger.effect.walk_mut(f);
                }
                UnitTrigger::TakeDamage(trigger) => {
                    trigger.effect.walk_mut(f);
                }
            }
        }
        for ability in self.abilities.values_mut() {
            ability.effect.walk_mut(f);
        }
    }
}

impl Default for UnitTemplate {
    fn default() -> Self {
        Self {
            hp: Health::new(1.0),
            spawn_animation_time: Time::new(0.0),
            speed: Coord::new(1.0),
            size: Coord::new(1.0),
            attack: AttackProperties {
                radius: Coord::new(1.0),
                cooldown: Time::new(1.0),
                animation_delay: Time::new(1.0),
                effect: default(),
            },
            triggers: default(),
            move_ai: MoveAi::Advance,
            target_ai: TargetAi::Closest,
            abilities: HashMap::new(),
            render_config: RenderConfig::Circle {
                color: Color::BLACK,
            },
            render_mode: RenderMode::Circle {
                color: Color::BLACK,
            },
            alliances: default(),
        }
    }
}

impl UnitTemplate {
    pub async fn load_render(
        &mut self,
        geng: &Geng,
        base_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        self.render_mode = match self.render_config {
            RenderConfig::Circle { color } => RenderMode::Circle { color },
            RenderConfig::Texture { ref path } => RenderMode::Texture {
                texture: geng::LoadAsset::load(&geng, &base_path.join(path)).await?,
            },
            RenderConfig::Shader { ref path } => RenderMode::Shader {
                program: geng::LoadAsset::load(&geng, &base_path.join(path)).await?,
            },
        };
        Ok(())
    }
}

impl geng::LoadAsset for UnitTemplate {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut result: Self = serde_json::from_str(&json)?;
            result.load_render(&geng, &path.parent().unwrap()).await?;
            Ok(result)
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

#[derive(Deref, Clone)]
pub struct UnitTemplates {
    #[deref]
    pub map: HashMap<String, UnitTemplate>,
}
