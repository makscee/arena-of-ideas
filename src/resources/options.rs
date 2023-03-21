use super::*;

#[derive(Deserialize, Debug)]
pub struct Options {
    pub fov: f32,
    pub log: HashMap<LogContext, bool>,
    pub shaders: Shaders,
    pub images: Images,
    pub colors: Colors,
    pub custom_game: CustomGameConfig,
    pub player_team_name: String,
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

#[derive(Deserialize, Debug)]
pub struct Shaders {
    pub unit: Shader,
    pub field: Shader,
    pub stats: Shader,
    pub strike: Shader,
    pub text: Shader,
    pub battle_timer: Shader,
    pub curve: Shader,
    pub name: Shader,
    pub slot: Shader,
    pub slot_price: Shader,
    pub money_indicator: Shader,
    pub status_panel: Shader,
    pub definitions_panel: Shader,
    pub status_panel_text: Shader,
    pub definitions_panel_text: Shader,
    pub definitions_panel_title: Shader,
    pub button: Shader,
    pub icon: Shader,
    pub corner_button: Shader,
}

#[derive(Deserialize, Debug)]
pub struct Images {
    pub eye_icon: Image,
    pub money_icon: Image,
    pub pool_icon: Image,
    pub play_icon: Image,
    pub pause_icon: Image,
    pub rewind_forward_icon: Image,
    pub rewind_backward_icon: Image,
    pub refresh_icon: Image,
}

#[derive(Deserialize, Debug)]
pub struct Colors {
    pub faction_colors: HashMap<Faction, Rgba<f32>>,
    pub inactive: Rgba<f32>,
    pub stats_attack_color: Rgba<f32>,
    pub stats_hp_color: Rgba<f32>,
    pub corner_button_color: Rgba<f32>,
    pub corner_button_icon_color: Rgba<f32>,
    pub btn_normal: Rgba<f32>,
    pub btn_active: Rgba<f32>,
    pub btn_hovered: Rgba<f32>,
    pub damage_text: Rgba<f32>,
}

#[derive(Deserialize, Debug)]
pub struct CustomGameConfig {
    pub enable: bool,
    pub light: Option<Team>,
    pub dark: Option<Team>,
}
