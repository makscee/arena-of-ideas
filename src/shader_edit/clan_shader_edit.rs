use crate::{assets::Assets, model::UnitTemplate};

use super::*;

pub fn run(geng: &Geng, assets: Assets) -> Box<dyn geng::State> {
    Box::new(ClanShaderState::new(geng, assets))
}

struct ClanShaderState {
    geng: Geng,
    time: Time,
    model: ClanShaderModel,
}

struct ClanShaderModel {
    units: HashMap<String, UnitTemplate>,
    shaders: HashMap<String, ClanShaderConfig>,
    selected_unit: String,
    selected_shader: String,
}

impl ClanShaderModel {
    pub fn new(assets: Assets) -> Self {
        let units: HashMap<String, UnitTemplate> = assets
            .units
            .map
            .into_iter()
            .map(|tuple| (tuple.0.clone(), tuple.1.clone()))
            .collect();
        let shaders: HashMap<String, ClanShaderConfig> = assets
            .clan_shaders
            .map
            .into_iter()
            .map(|tuple| (tuple.0.clone(), tuple.1.clone()))
            .collect();
        Self {
            selected_unit: units
                .keys()
                .next()
                .expect("Must be at least one unit to edit")
                .clone(),
            selected_shader: shaders
                .keys()
                .next()
                .expect("Must be at least one clan shader")
                .clone(),
            units,
            shaders,
        }
    }
}

impl ClanShaderState {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        Self {
            model: ClanShaderModel::new(assets),
            geng: geng.clone(),
            time: Time::ZERO,
        }
    }

    pub fn save(self) {
        if let Some(unit) = self.model.units.get(&self.model.selected_unit) {
            let data = serde_json::to_string_pretty(&unit.path).expect("Failed to serialize item");
            std::fs::write(&unit.path, data)
                .expect(&format!("Cannot save _list: {:?}", &unit.path));
        }
    }
}

impl geng::State for ClanShaderState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.time += delta_time;
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {}
}
