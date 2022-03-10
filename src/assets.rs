use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub units: UnitTemplates,
    pub config: Config,
}

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
                    let template = <UnitTemplate as geng::LoadAsset>::load(
                        &geng,
                        &base_path.join(format!("{}.json", typ)),
                    );
                    map.insert(typ, template.await?);
                }
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}
