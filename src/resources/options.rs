use geng::prelude::serde_json::Value;

use super::*;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub shaders: Shaders,
    pub images: Images,
    pub colors: Colors,
    pub floats: Floats,
    pub uniforms: Uniforms,
    pub parameters: Parameters,

    pub fov: f32,
    pub rewind_speed: f32,
    pub rewind_add_speed: f32,
    pub initial_shop_fill: usize,
    pub log: HashMap<LogContext, bool>,
    pub custom_game: CustomGameConfig,
    pub rate_heroes: bool,
    pub generate_ladder: bool,
    pub player_team_name: String,
    pub initial_shop_g: i32,
    pub initial_shop_slots: usize,
    pub initial_team_slots: usize,
    pub initial_state: GameState,
    pub shop_max_slots: usize,
}

use std::cell::RefCell;
thread_local!(pub static OPTIONS_COLORS: RefCell<HashMap<String, Rgba<f32>>> = RefCell::new(HashMap::default()));

impl FileWatcherLoader for Options {
    fn load(resources: &mut Resources, _: &PathBuf, _: &mut FileWatcherSystem) {
        resources.options = Self::do_load();
        resources.transition_state = GameState::MainMenu;
    }
}

impl Options {
    pub fn do_load() -> Options {
        let value: Value =
            futures::executor::block_on(load_json(static_path().join("options.json"))).unwrap();
        let map = value
            .as_object()
            .unwrap()
            .get("colors")
            .unwrap()
            .clone()
            .as_object()
            .unwrap()
            .clone();
        let mut colors: HashMap<String, Rgba<f32>> = default();
        for (key, value) in map {
            if let Some(value) = value.as_str() {
                if let Ok(value) = Rgba::try_from(value) {
                    colors.insert(key, value);
                }
            }
        }
        OPTIONS_COLORS.with(|map| {
            *map.borrow_mut() = colors;
        });
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Shaders {
    pub panel_header: ShaderChain,
    pub panel_footer: ShaderChain,
    pub panel_body: ShaderChain,
    pub panel_button: ShaderChain,
    pub panel_text: ShaderChain,
    pub unit: ShaderChain,
    pub unit_card: ShaderChain,
    pub field: ShaderChain,
    pub stats: ShaderChain,
    pub strike: ShaderChain,
    pub text: ShaderChain,
    pub battle_timer: ShaderChain,
    pub curve: ShaderChain,
    pub name: ShaderChain,
    pub slot: ShaderChain,
    pub slot_sacrifice_marker: ShaderChain,
    pub button: ShaderChain,
    pub slot_button: ShaderChain,
    pub button_icon: ShaderChain,
    pub icon: ShaderChain,
    pub team_name: ShaderChain,
    pub team_name_intro: ShaderChain,
    pub tape_indicator: ShaderChain,
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
    pub shop: Rgba<f32>,
    pub player: Rgba<f32>,
    pub enemy: Rgba<f32>,
    pub text: Rgba<f32>,
    pub sacrifice: Rgba<f32>,
    pub primary: Rgba<f32>,
    pub secondary: Rgba<f32>,
    pub button: Rgba<f32>,
    pub outline: Rgba<f32>,
    pub active: Rgba<f32>,
    pub inactive: Rgba<f32>,
    pub hovered: Rgba<f32>,
    pub pressed: Rgba<f32>,
    pub add: Rgba<f32>,
    pub subtract: Rgba<f32>,
    pub common: Rgba<f32>,
    pub rare: Rgba<f32>,
    pub epic: Rgba<f32>,
    pub legendary: Rgba<f32>,
    pub damage: Rgba<f32>,
    pub healing: Rgba<f32>,
    pub victory: Rgba<f32>,
    pub defeat: Rgba<f32>,
    pub stat_hp: Rgba<f32>,
    pub stat_atk: Rgba<f32>,
    pub light: Rgba<f32>,
    pub dark: Rgba<f32>,
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
    pub panel_text_padding: f32,
    pub panel_card_padding: f32,
    pub panel_row_padding: f32,
    pub panel_row_spacing: f32,
    pub panel_row_index_offset: vec2<f32>,
    pub panel_column_padding: f32,
    pub panel_column_spacing: f32,
}

#[derive(Deserialize, Debug)]
pub struct Uniforms {
    pub ui_button: ShaderUniforms,
}

#[derive(Deserialize, Debug)]
pub struct Parameters {
    pub panels: HashMap<PanelType, ShaderParameters>,
    pub panel_card: ShaderParameters,
}
