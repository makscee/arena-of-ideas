use super::*;

#[derive(Deserialize, Debug)]
pub struct Options {
    pub fov: f32,
    pub field: Shader,
    pub stats: Shader,
    pub stats_attack_color: Rgba<f32>,
    pub stats_hp_color: Rgba<f32>,
    pub strike: Shader,
}

impl geng::LoadAsset for Options {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let options: Options = serde_json::from_str(&json)?;
            Ok(options)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some(".json");
}
