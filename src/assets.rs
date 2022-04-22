use super::*;

#[derive(Deserialize, geng::Assets)]
#[asset(json)]
pub struct Options {
    pub alliance_colors: HashMap<Alliance, Color<f32>>,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub units: UnitTemplates,
    pub options: Options,
    pub textures: Textures,
    pub shaders: Shaders,
}

pub type Key = String;
pub type Wave = HashMap<String, Vec<UnitType>>;

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

#[derive(geng::Assets, Deserialize, Clone)]
#[asset(json)]
pub struct Config {
    pub player: Vec<UnitType>,
    pub alliances: HashMap<Alliance, usize>,
    pub spawn_points: HashMap<String, Vec2<Coord>>,
    pub waves: Vec<Wave>,
    pub fov: f32,
}

impl Assets {
    pub fn get_render(&self, config: &RenderConfig) -> RenderMode {
        match config {
            &RenderConfig::Circle { color } => RenderMode::Circle { color },
            RenderConfig::Texture { path } => RenderMode::Texture {
                texture: self
                    .textures
                    .get(path)
                    .expect(&format!(
                        "Unknown texture: {path:?}. Perhaps you need to add it in textures.json"
                    ))
                    .clone(),
            },
            RenderConfig::Shader { path } => RenderMode::Shader {
                program: self
                    .shaders
                    .get(path)
                    .expect(&format!(
                        "Unknown shader: {path:?}. Perhaps you need to add it in shaders.json"
                    ))
                    .clone(),
            },
        }
    }
}

impl geng::LoadAsset for UnitTemplates {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            geng.shader_lib().add(
                "common.glsl",
                &<String as geng::LoadAsset>::load(
                    &geng,
                    &path.parent().unwrap().join("common.glsl"),
                )
                .await?,
            );
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let packs: Vec<String> = serde_json::from_str(&json)?;
            let mut map = HashMap::new();
            for pack in packs {
                let base_path = path.parent().unwrap().join(pack);
                let json =
                    <String as geng::LoadAsset>::load(&geng, &base_path.join("_list.json")).await?;
                let types: Vec<String> = serde_json::from_str(&json)?;
                for typ in types {
                    let mut json = <serde_json::Value as geng::LoadAsset>::load(
                        &geng,
                        &base_path.join(format!("{}.json", typ)),
                    )
                    .await?;
                    if let Some(base) = json.get_mut("base") {
                        let base = base.take();
                        let base = base.as_str().expect("base must be a string");
                        let base = &map[base];
                        let mut base_json = serde_json::to_value(base).unwrap();
                        base_json
                            .as_object_mut()
                            .unwrap()
                            .append(&mut json.as_object_mut().unwrap());
                        json = base_json;
                        json.as_object_mut().unwrap().remove("base");
                    }
                    let mut template: UnitTemplate = serde_json::from_value(json)?;
                    info!(
                        "{:?} => {}",
                        typ,
                        serde_json::to_string_pretty(&template).unwrap()
                    );
                    map.insert(typ, template);
                }
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
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
