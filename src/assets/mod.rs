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

    Ok::<_, anyhow::Error>(system_shaders)
}
