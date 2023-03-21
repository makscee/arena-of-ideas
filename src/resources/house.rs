use super::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct House {
    pub name: HouseName,
    pub color: Rgba<f32>,
    pub abilities: HashMap<String, Ability>,
    pub statuses: HashMap<String, Status>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum HouseName {
    Vampires,
    Dragons,
    Robots,
    Snakes,
}

impl geng::LoadAsset for House {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut house: House = serde_json::from_str(&json)?;
            house.statuses.iter_mut().for_each(|(_, status)| {
                status.color = Some(house.color);
            });
            Ok(house)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some(".json");
}
