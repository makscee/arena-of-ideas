use super::*;

#[derive(Deserialize, Clone)]
pub struct Round {
    pub name: String,
    pub enemies: Vec<String>,
}

impl geng::LoadAsset for Round {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let round: Round = serde_json::from_str(&json)?;
            Ok(round)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some(".json");
}
