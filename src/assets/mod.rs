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

    let path = system_shaders.field.path.clone();
    let program = <ugli::Program as geng::LoadAsset>::load(&geng, &base_path.join(path.clone()))
        .await
        .context(format!("Failed to load {path}"))?;
    system_shaders.field.program = Some(Rc::new(program));

    let path = system_shaders.unit.path.clone();
    let program = <ugli::Program as geng::LoadAsset>::load(&geng, &base_path.join(path.clone()))
        .await
        .context(format!("Failed to load {path}"))?;
    system_shaders.unit.program = Some(Rc::new(program));

    Ok::<_, anyhow::Error>(system_shaders)
}
