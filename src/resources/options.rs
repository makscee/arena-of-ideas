use super::*;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub shaders: Shaders,
    pub images: Images,
    pub colors: Colors,
    pub floats: Floats,
    pub widgets: Widgets,
    pub uniforms: Uniforms,

    pub fov: f32,
    pub rewind_speed: f32,
    pub rewind_add_speed: f32,
    pub initial_shop_fill: usize,
    pub log: HashMap<LogContext, bool>,
    pub custom_game: CustomGameConfig,
    pub walkthrough: bool,
    pub team_ratings: bool,
    pub player_team_name: String,
    pub initial_shop_g: i32,
    pub initial_shop_slots: usize,
    pub initial_team_slots: usize,
    pub initial_state: GameState,
    pub shop_max_slots: usize,
}

impl FileWatcherLoader for Options {
    fn load(resources: &mut Resources, _: &PathBuf, _: &mut FileWatcherSystem) {
        resources.options = Self::do_load();
        resources.transition_state = GameState::MainMenu;
    }
}

impl Options {
    pub fn do_load() -> Options {
        futures::executor::block_on(load_json(static_path().join("options.json"))).unwrap()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Shaders {
    pub unit: Shader,
    pub unit_card: Shader,
    pub field: Shader,
    pub stats: Shader,
    pub strike: Shader,
    pub text: Shader,
    pub battle_timer: Shader,
    pub curve: Shader,
    pub name: Shader,
    pub slot: Shader,
    pub slot_price: Shader,
    pub slot_sacrifice_marker: Shader,
    pub money_indicator: Shader,
    pub status_panel: Shader,
    pub definitions_panel: Shader,
    pub status_panel_text: Shader,
    pub definitions_panel_text: Shader,
    pub definitions_panel_title: Shader,
    pub button: Shader,
    pub button_icon: Shader,
    pub icon: Shader,
    pub corner_button: Shader,
    pub team_name: Shader,
    pub team_name_intro: Shader,
    pub tape_indicator: Shader,
    pub shop_sell_field: Shader,
    pub battle_score_indicator: Shader,
    pub state_indicator: Shader,
    pub stars_indicator: Shader,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Widgets {
    pub battle_choice_panel: BattleChoicePanel,
    pub battle_over_panel: BattleOverPanel,
    pub bonus_choice_panel: BonusChoicePanel,
    pub card_choice_panel: CardChoicePanel,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CardChoicePanel {
    pub panel: AnimatedShader,
    pub card_bg: AnimatedShader,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BattleChoicePanel {
    pub bg: AnimatedShader,
    pub star: Shader,
    pub name: Shader,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BattleOverPanel {
    pub bg_1: AnimatedShader,
    pub bg_2: AnimatedShader,
    pub title: AnimatedShader,
    pub star: AnimatedShader,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BonusChoicePanel {
    pub bg: AnimatedShader,
    pub option: AnimatedShader,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AnimatedShader {
    pub shader: Shader,
    pub animation: AnimatedShaderUniforms,
    pub duration: Time,
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
    pub healing: Rgba<f32>,
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
    pub slots_striker_position: vec2<f32>,
    pub slots_battle_team_spacing: vec2<f32>,
    pub slots_shop_spacing: vec2<f32>,
    pub slots_battle_team_scale: f32,
    pub slots_striker_scale: f32,
    pub slot_info_offset: f32,
}

#[derive(Deserialize, Debug)]
pub struct Uniforms {
    pub ui_button: ShaderUniforms,
}
