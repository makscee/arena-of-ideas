use std::path::PathBuf;

use super::*;

mod ability;
mod clan;
mod round;
mod unit_template;

pub use ability::*;
pub use clan::*;
pub use round::*;
pub use unit_template::*;

#[derive(geng::Assets)]
pub struct Assets {
    // pub units: Vec<UnitTemplate>,
    // pub clans: Vec<Clan>,
    // pub rounds: Vec<Round>,
    #[asset(load_with = "load_system_shaders(geng, &base_path)")]
    pub system_shaders: SystemShaders,
    #[asset(load_with = "load_shader_library(geng, &base_path)")]
    pub shader_library: Vec<PathBuf>,
}

async fn load_system_shaders(
    geng: &Geng,
    base_path: &std::path::Path,
) -> anyhow::Result<SystemShaders> {
    let base_path = base_path.join("shaders/system/");
    let json = <String as geng::LoadAsset>::load(geng, &base_path.join("config.json"))
        .await
        .context("Failed to load config.json for system shaders")?;
    let mut system_shaders: SystemShaders =
        serde_json::from_str(&json).context("Failed to parse config.json for system shaders")?;

    system_shaders.field.load(&geng).await;
    system_shaders.unit.load(&geng).await;

    Ok::<_, anyhow::Error>(system_shaders)
}

pub async fn load_shader_library(
    geng: &Geng,
    base_path: &std::path::Path,
) -> anyhow::Result<Vec<PathBuf>> {
    // load shader library
    let shader_library_list =
        <String as geng::LoadAsset>::load(&geng, &static_path().join("shaders/library/_list.json"))
            .await
            .context("Failed to load shader library list")?;
    let shader_library_list: Vec<String> = serde_json::from_str(&shader_library_list)
        .context("Failed to parse shader library list")?;
    let shader_library_list: Vec<PathBuf> = shader_library_list
        .iter()
        .map(|path| static_path().join("shaders/library/").join(path))
        .collect();
    for path in shader_library_list.iter() {
        let asset_path = static_path().join("shaders/library").join(&path);
        geng.shader_lib().add(
            path.to_str().expect("Failed to get path"),
            &<String as geng::LoadAsset>::load(&geng, &asset_path)
                .await
                .context(format!("Failed to load {:?}", asset_path))?,
        );
    }

    Ok::<_, anyhow::Error>(shader_library_list)
}
