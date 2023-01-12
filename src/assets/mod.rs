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
    #[asset(path = "units/_list.json", load_with = "load_units(geng, &base_path)")]
    pub units: Vec<UnitTemplate>,
    #[asset(path = "clans/_list.json", load_with = "load_clans(geng, &base_path)")]
    pub clans: Vec<Clan>,
    #[asset(path = "rounds/round*.json", range = "1..=10")]
    pub rounds: Vec<Round>,
    #[asset(load_with = "load_shader_library(geng, &base_path)")]
    pub shader_library: Vec<PathBuf>,
    #[asset(load_with = "load_system_shaders(geng, &base_path)")]
    pub system_shaders: SystemShaders,
}

async fn load_units(geng: &Geng, base_path: &std::path::Path) -> anyhow::Result<Vec<UnitTemplate>> {
    let json = <String as geng::LoadAsset>::load(&geng, &base_path).await?;
    let paths: Vec<String> = serde_json::from_str(&json)?;
    let mut result: Vec<UnitTemplate> = vec![];
    for path in paths {
        let path = base_path.join(path).join(".json");
        let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
        let mut asset: UnitTemplate =
            serde_json::from_str(&json).context(format!("Failed to parse from {path:?}"))?;

        let path = base_path.join(path).join("_render.json");
        let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
        let clan_renders: Vec<Vec<ShaderConfig>> =
            serde_json::from_str(&json).context(format!("Failed to parse from {path:?}"))?;
        asset.clan_renders = clan_renders;
        result.push(asset);
    }
    Ok(result)
}

async fn load_clans(geng: &Geng, base_path: &std::path::Path) -> anyhow::Result<Vec<Clan>> {
    let json = <String as geng::LoadAsset>::load(&geng, &base_path).await?;
    let paths: Vec<std::path::PathBuf> = serde_json::from_str(&json)?;
    let mut result: Vec<Clan> = vec![];
    for path in paths {
        let path = base_path.join(path);
        let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
        let asset: Clan =
            serde_json::from_str(&json).context(format!("Failed to parse from {path:?}"))?;
        result.push(asset);
    }
    Ok(result)
}

pub async fn load_system_shaders(
    geng: &Geng,
    base_path: &std::path::Path,
) -> anyhow::Result<SystemShaders> {
    debug!("Loading system shaders");
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
    debug!("Loading shader library");
    let base_path = base_path.join("shaders/library/");
    let shader_library_list =
        <String as geng::LoadAsset>::load(&geng, &base_path.join("_list.json"))
            .await
            .context("Failed to load shader library list")?;
    let shader_library_list: Vec<String> = serde_json::from_str(&shader_library_list)
        .context("Failed to parse shader library list")?;
    let shader_library_list: Vec<PathBuf> = shader_library_list
        .iter()
        .map(|path| base_path.join(path))
        .collect();
    for path in shader_library_list.iter() {
        let asset_path = base_path.join(&path);
        debug!("Add to shader library {}", path.to_str().unwrap());
        geng.shader_lib().add(
            path.file_name()
                .unwrap()
                .to_str()
                .expect("Failed to get shader path"),
            &<String as geng::LoadAsset>::load(&geng, &asset_path)
                .await
                .context(format!("Failed to load {:?}", asset_path))?,
        );
    }

    Ok::<_, anyhow::Error>(shader_library_list)
}
