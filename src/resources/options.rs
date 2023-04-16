use super::*;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub shaders: Shaders,
    pub images: Images,
    pub colors: Colors,
    pub floats: Floats,
    pub fov: f32,
    pub rewind_speed: f32,
    pub rewind_add_speed: f32,
    pub initial_shop_fill: usize,
    pub log: HashMap<LogContext, bool>,
    pub custom_game: CustomGameConfig,
    pub walkthrough: bool,
    pub team_ratings: bool,
    pub player_team_name: String,
    pub initial_shop_slots: usize,
    pub initial_team_slots: usize,
}

impl FileWatcherLoader for Options {
    fn loader(resources: &mut Resources, _: &PathBuf, _: &mut FileWatcherSystem) {
        resources.options = Self::load();
        resources.transition_state = GameState::MainMenu;
    }
}

impl Options {
    pub fn load() -> Options {
        futures::executor::block_on(load_json(static_path().join("options.json"))).unwrap()
    }
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
    pub button_text: Shader,
    pub button_icon: Shader,
    pub icon: Shader,
    pub corner_button: Shader,
    pub team_name: Shader,
    pub team_name_intro: Shader,
    pub tape_indicator: Shader,
    pub shop_sell_field: Shader,
    pub battle_score_indicator: Shader,
    pub battle_over_panel_bg: Shader,
    pub battle_over_panel_title: Shader,
    pub battle_over_panel_star: Shader,
    pub choice_panel: Shader,
    pub choice_panel_option: Shader,
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
    pub factions: HashMap<Faction, Rgba<f32>>,
    pub rarities: HashMap<Rarity, Rgba<f32>>,
    pub background_light: Rgba<f32>,
    pub background_dark: Rgba<f32>,
    pub inactive: Rgba<f32>,
    pub active: Rgba<f32>,
    pub hovered: Rgba<f32>,
    pub pressed: Rgba<f32>,
    pub outline: Rgba<f32>,
    pub primary: Rgba<f32>,
    pub secondary: Rgba<f32>,
    pub accent: Rgba<f32>,
    pub background: Rgba<f32>,
    pub stats_attack: Rgba<f32>,
    pub stats_health: Rgba<f32>,
    pub damage: Rgba<f32>,
    pub addition: Rgba<f32>,
    pub deletion: Rgba<f32>,
    pub victory: Rgba<f32>,
    pub defeat: Rgba<f32>,
    pub star: Rgba<f32>,
    pub button: Rgba<f32>,
    pub text: Rgba<f32>,
}

#[derive(Deserialize, Debug)]
pub struct CustomGameConfig {
    pub enable: bool,
    pub light: Option<ReplicatedTeam>,
    pub dark: Option<ReplicatedTeam>,
}

#[derive(Deserialize, Debug)]
pub struct Floats {
    pub slots_battle_team_position: vec2<f32>,
    pub slots_shop_team_position: vec2<f32>,
    pub slots_sacrifice_position: vec2<f32>,
    pub slots_striker_position: vec2<f32>,
    pub slots_battle_team_spacing: vec2<f32>,
    pub slots_shop_spacing: vec2<f32>,
    pub slots_sacrifice_spacing: vec2<f32>,
    pub slots_battle_team_scale: f32,
    pub slots_striker_scale: f32,
    pub slot_info_offset: f32,
}
