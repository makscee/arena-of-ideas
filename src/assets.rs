use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub units: UnitTemplates,
}

pub type Key = String;
pub type Wave = HashMap<String, Vec<UnitType>>;

#[derive(geng::Assets, Deserialize, Clone)]
#[asset(json)]
pub struct Config {
    pub player: Vec<UnitType>,
    pub spawn_points: HashMap<String, Vec2<Coord>>,
    pub waves: Vec<Wave>,
}

impl geng::LoadAsset for UnitTemplates {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
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
                    }
                    let mut template: UnitTemplate = serde_json::from_value(json)?;
                    template.load_render(&geng, &base_path).await?;
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
