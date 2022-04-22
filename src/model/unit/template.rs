use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(skip)]
    pub render_mode: RenderMode,
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
            render_mode: RenderMode::Circle {
                color: Color::WHITE,
            },
            alliances: default(),
        }
    }
}

impl RenderMode {
    pub async fn load(
        config: &RenderConfig,
        geng: &Geng,
        base_path: &std::path::Path,
    ) -> anyhow::Result<Self> {
        match config {
            &RenderConfig::Circle { color } => Ok(RenderMode::Circle { color }),
            RenderConfig::Texture { path } => Ok(RenderMode::Texture {
                texture: {
                    let path = std::path::Path::new(path);
                    match path.extension().and_then(|s| s.to_str()) {
                        Some("svg") => {
                            let data =
                                <String as geng::LoadAsset>::load(geng, &base_path.join(path))
                                    .await?;
                            let tree = usvg::Tree::from_data(
                                data.as_bytes(),
                                &usvg::Options::default().to_ref(),
                            )?;
                            let mut pixmap = tiny_skia::Pixmap::new(
                                tree.svg_node().size.width().ceil() as _,
                                tree.svg_node().size.height().ceil() as _,
                            )
                            .unwrap();
                            resvg::render(
                                &tree,
                                usvg::FitTo::Original,
                                tiny_skia::Transform::identity(),
                                pixmap.as_mut(),
                            );
                            let texture = ugli::Texture::new_with(
                                geng.ugli(),
                                vec2(pixmap.width() as usize, pixmap.height() as usize),
                                |pos| {
                                    let color = pixmap
                                        .pixel(pos.x as u32, pixmap.height() - 1 - pos.y as u32)
                                        .unwrap();
                                    Color::rgba(
                                        color.red(),
                                        color.green(),
                                        color.blue(),
                                        color.alpha(),
                                    )
                                    .convert()
                                },
                            );
                            Rc::new(texture)
                        }
                        _ => geng::LoadAsset::load(&geng, &base_path.join(path)).await?,
                    }
                },
            }),
            RenderConfig::Shader { ref path } => Ok(RenderMode::Shader {
                program: geng::LoadAsset::load(&geng, &base_path.join(path)).await?,
            }),
        }
    }
}

impl UnitTemplate {
    pub async fn load_render(
        &mut self,
        geng: &Geng,
        base_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        self.render_mode = RenderMode::load(&self.render_config, geng, base_path).await?;
        self.walk_effects_mut(&mut |effect| match effect {
            Effect::Visual(effect) => {
                effect.render_mode =
                    RenderMode::load(&effect.render_config, geng, base_path).await?;
            }
            _ => {}
        });
        Ok(())
    }
}

// impl geng::LoadAsset for UnitTemplate {
//     fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
//         let geng = geng.clone();
//         let path = path.to_owned();
//         async move {
//             let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
//             let mut result: Self = serde_json::from_str(&json)?;
//             result.load_render(&geng, &path.parent().unwrap()).await?;
//             Ok(result)
//         }
//         .boxed_local()
//     }
//     const DEFAULT_EXT: Option<&'static str> = Some("json");
// }

#[derive(Deref, DerefMut, Clone)]
pub struct UnitTemplates {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, UnitTemplate>,
}
