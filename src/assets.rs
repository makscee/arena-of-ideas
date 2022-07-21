use std::collections::VecDeque;

use super::*;

use once_cell::sync::Lazy;

#[derive(Deserialize, geng::Assets)]
#[asset(json)]
pub struct Options {
    pub clan_colors: HashMap<Clan, Color<f32>>,
}

// Used because deserializing with state is not as trivial as writing
// `#[derive(Deserialize)]`, and requires too much boilerplate.
pub static EFFECT_PRESETS: Lazy<Mutex<Effects>> =
    Lazy::new(|| Mutex::new(Effects { map: default() }));

#[derive(Deserialize, Clone)]
// #[serde(deny_unknown_fields)]
pub struct StatusConfig {
    #[serde(flatten)]
    pub status: Status,
    pub render: Option<ShaderConfig>,
}

#[derive(Deref, DerefMut, Clone)]
pub struct Statuses {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, StatusConfig>,
}

impl Statuses {
    pub fn get_config(&self, status_name: &StatusName) -> &StatusConfig {
        self.get(status_name)
            .expect(&format!("Failed to get status {status_name}"))
    }
}

#[derive(geng::Assets)]
pub struct Assets {
    pub units: UnitTemplates,
    pub statuses: Statuses,
    #[asset(load_with = "load_field_render(geng, &base_path)")]
    pub field_render: ShaderProgram,
    pub clans: ClanEffects,
    pub options: Options,
    pub textures: Textures,
    pub shaders: Shaders,
    pub card: Rc<ugli::Texture>,
    #[asset(path = "rounds/round*.json", range = "1..=5")]
    pub rounds: Vec<GameRound>,
}

async fn load_field_render(
    geng: &Geng,
    base_path: &std::path::Path,
) -> anyhow::Result<ShaderProgram> {
    let json = <String as geng::LoadAsset>::load(geng, &base_path.join("field_render.json"))
        .await
        .context("Failed to load field_render.json")?;
    let config: ShaderConfig =
        serde_json::from_str(&json).context("Failed to parse field_render.json")?;
    let path = config.path.as_str();
    let program = <ugli::Program as geng::LoadAsset>::load(&geng, &static_path().join(path))
        .await
        .context(format!("Failed to load {path}"))?;
    let result = ShaderProgram {
        program: Rc::new(program),
        parameters: config.parameters,
        vertices: 2,
        instances: 1,
    };
    Ok::<_, anyhow::Error>(result)
}

pub type Key = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GameRound {
    #[serde(default)]
    pub statuses: Vec<StatusRef>,
    #[serde(default)]
    pub enemies: Vec<UnitType>,
}

#[derive(Deref, DerefMut)]
pub struct Textures {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, Rc<ugli::Texture>>,
}

#[derive(Deref, DerefMut)]
pub struct Shaders {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, Rc<ugli::Program>>,
}

#[derive(Deref, DerefMut)]
pub struct Effects {
    #[deref]
    #[deref_mut]
    pub map: HashMap<String, Effect>,
}

#[derive(geng::Assets, Deserialize, Clone)]
#[asset(json)]
pub struct Config {
    #[serde(default)]
    pub player: Vec<UnitType>,
    #[serde(default)]
    pub clans: HashMap<Clan, usize>,
    pub fov: f32,
}

#[derive(Debug, Deserialize, geng::Assets, Clone)]
#[asset(json)]
pub struct ShopRenderConfig {
    pub clans: HashMap<Clan, ClanRenderConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClanRenderConfig {
    pub description: String,
    pub rows: usize,
    pub columns: usize,
}

#[derive(geng::Assets, Clone)]
pub struct ShopConfig {
    pub render: ShopRenderConfig,
}

impl Default for ShopConfig {
    fn default() -> Self {
        Self {
            render: ShopRenderConfig { clans: default() },
        }
    }
}

#[derive(Clone, Deref)]
pub struct ClanEffects {
    #[deref]
    pub map: HashMap<Clan, Vec<ClanEffect>>,
}

impl Assets {
    pub fn get_render(&self, config: &ShaderConfig) -> ShaderProgram {
        ShaderProgram {
            program: self
                .shaders
                .get(&config.path)
                .expect(&format!(
                    "Unknown shader: {:?}. Perhaps you need to add it in shaders.json",
                    config.path
                ))
                .clone(),
            parameters: config.parameters.clone(), // TODO: avoid cloning
            vertices: config.vertices,
            instances: config.instances,
        }
    }
}

impl geng::LoadAsset for GameRound {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let mut json = <serde_json::Value as geng::LoadAsset>::load(&geng, &path).await?;
            if let Some(preset) = json.get_mut("preset") {
                // Load preset
                let preset = preset.take();
                let preset = preset.as_str().expect("preset must be a string");
                let preset = <String as geng::LoadAsset>::load(
                    &geng,
                    &path.as_path().parent().unwrap().join(preset),
                )
                .await?;
                let mut preset: GameRound = serde_json::from_str(&preset)?;

                // Parse round
                json.as_object_mut().unwrap().remove("preset");
                let round: GameRound = serde_json::from_value(json)?;

                // Append statuses
                preset.statuses.extend(round.statuses);
                return Ok(dbg!(preset));
            }
            let round: GameRound = serde_json::from_value(json)?;
            Ok(round)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

impl geng::LoadAsset for UnitTemplates {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let shader_library_list = <String as geng::LoadAsset>::load(
                &geng,
                &static_path().join("shader_library/_list.json"),
            )
            .await
            .context("Failed to load shader library list")?;
            let shader_library_list: Vec<String> = serde_json::from_str(&shader_library_list)
                .context("Failed to parse shader library list")?;
            for path in shader_library_list {
                let asset_path = static_path().join("shader_library").join(&path);
                geng.shader_lib().add(
                    path.as_str(),
                    &<String as geng::LoadAsset>::load(&geng, &asset_path)
                        .await
                        .context(format!("Failed to load {:?}", asset_path))?,
                );
            }

            let json = <String as geng::LoadAsset>::load(&geng, &path)
                .await
                .context(format!("Failed to load unit json from {:?}", path))?;
            let packs: Vec<String> = serde_json::from_str(&json)?;
            let mut map = HashMap::new();
            for pack in packs {
                let base_path = path.parent().unwrap().join(pack);
                let path = base_path.join("_list.json");
                let json = <String as geng::LoadAsset>::load(&geng, &path)
                    .await
                    .context(format!("Failed to load {path:?}"))?;
                let types: Vec<String> = serde_json::from_str(&json)?;
                for typ in types {
                    let path = base_path.join(format!("{}.json", typ));
                    let mut json = <serde_json::Value as geng::LoadAsset>::load(&geng, &path)
                        .await
                        .context(format!("Failed to load {path:?}"))?;
                    if let Some(base) = json.get_mut("base") {
                        let base = base.take();
                        let base = base.as_str().expect("base must be a string");
                        let base_str = base.to_string();
                        let base = &map
                            .get(base)
                            .expect(&format!("Failed to find unit's base: {}", base));
                        let mut base_json = serde_json::to_value(base).unwrap();
                        base_json
                            .as_object_mut()
                            .unwrap()
                            .append(&mut json.as_object_mut().unwrap());
                        json = base_json;
                        let description = json
                            .get("description")
                            .map(|description| description.to_string().trim_matches('"').to_owned())
                            .unwrap_or_default();
                        json.as_object_mut().unwrap().insert(
                            String::from("description"),
                            serde_json::Value::String(format!("{}\n{}", base_str, description)),
                        );
                        json.as_object_mut().unwrap().remove("base");
                    }

                    let template: UnitTemplate = serde_json::from_value(json)
                        .context(format!("Failed to parse {path:?}"))?;

                    // info!(
                    //     "{:?} => {}",
                    //     typ,
                    //     serde_json::to_string_pretty(&template).unwrap()
                    // );
                    map.insert(typ, template);
                }
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

impl geng::LoadAsset for Statuses {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let base_path = path.parent().unwrap().join("statuses");
            let path = base_path.join("_list.json");
            let json = <String as geng::LoadAsset>::load(&geng, &path)
                .await
                .context(format!("Failed to load list of statuses from {path:?}"))?;
            let paths: Vec<std::path::PathBuf> = serde_json::from_str(&json)
                .context(format!("Failed to parse list of statuses from {path:?}"))?;
            let mut map = HashMap::new();
            for path in paths {
                let path = base_path.join(path);
                let json = <String as geng::LoadAsset>::load(&geng, &path)
                    .await
                    .context(format!("Failed to load status from {path:?}"))?;
                let config: StatusConfig = serde_json::from_str(&json)
                    .context(format!("Failed to parse status from {path:?}"))?;

                map.insert(config.status.name.clone(), config);
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}

impl geng::LoadAsset for ClanEffects {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let base_path = path.parent().unwrap().join("clan_effects");
            let path = base_path.join("_list.json");
            let json = <String as geng::LoadAsset>::load(&geng, &path)
                .await
                .context(format!("Failed to load list of clan effects from {path:?}"))?;
            let paths: HashMap<Clan, std::path::PathBuf> = serde_json::from_str(&json).context(
                format!("Failed to parse list of clan effects from {path:?}"),
            )?;
            let mut map = HashMap::new();
            for (clan, path) in paths {
                let path = base_path.join(path);
                let json = <String as geng::LoadAsset>::load(&geng, &path)
                    .await
                    .context(format!(
                        "Failed to load clan ({clan:?}) effects from {path:?}"
                    ))?;
                let effects: Vec<ClanEffect> = serde_json::from_str(&json)
                    .context(format!("Failed to parse clan effects from {path:?}"))?;
                map.insert(clan, effects);
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}

impl geng::LoadAsset for Shaders {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let base_path = path.parent().unwrap();
            let shaders: Vec<String> = serde_json::from_str(&json)?;
            let mut map = HashMap::new();
            for path in shaders {
                let shader_path = base_path.join(&path);
                let shader = geng::LoadAsset::load(&geng, &shader_path).await?;
                map.insert(path, shader);
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

impl geng::LoadAsset for Textures {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let base_path = path.parent().unwrap();
            let textures: Vec<String> = serde_json::from_str(&json)?;
            let mut map = HashMap::new();
            for path in textures {
                let texture_path = base_path.join(&path);
                let texture = match texture_path.extension().and_then(|s| s.to_str()) {
                    Some("svg") => {
                        let data = <String as geng::LoadAsset>::load(&geng, &texture_path).await?;
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
                                Color::rgba(color.red(), color.green(), color.blue(), color.alpha())
                                    .convert()
                            },
                        );
                        Rc::new(texture)
                    }
                    _ => geng::LoadAsset::load(&geng, &texture_path).await?,
                };
                map.insert(path, texture);
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

impl Effects {
    pub fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<()> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let base_path = path.parent().unwrap();
            let effects: Vec<String> = serde_json::from_str(&json)?;
            for path in effects {
                let effect_path = base_path.join(&path);
                let json = <String as geng::LoadAsset>::load(&geng, &effect_path).await?;
                let effect = serde_json::from_str(&json)?;
                EFFECT_PRESETS.lock().unwrap().insert(path, effect);
            }
            Ok(())
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}
