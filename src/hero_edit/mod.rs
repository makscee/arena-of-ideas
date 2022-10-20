use std::{array, env};

use geng::prelude::*;
use geng::prelude::{itertools::Itertools, ugli::raw::RGBA};

use crate::{
    render::UnitRender,
    shader_edit::{ClanShaderParam, ClanShaderType},
};

use super::*;

#[derive(clap::Args)]
pub struct HeroEditor {}

impl HeroEditor {
    fn draw_units_widget(units: Vec<&String>) {
        // units.iter().for_each();
        // let value = Value::from(*units.first().expect("No units"));
        // let args = vmap! {
        //     "type" => "Enum",
        //     "name" => "Select unit",
        //     "id" => "unit",
        //     "values" => values,
        //     "value" => value,
        // };
        // panel
        //     .call_method("createWidget", &[args])
        //     .expect("Error while calling createWidget()");
    }

    // fn create_widget(panel: Element, param: ClanShaderParam) {
    //     let mut args;
    //     match param.value {
    //         shader_edit::ClanShaderType::Enum { values, show_all } => {
    //             let param_values = values;
    //             let mut values = Value::new();
    //             param_values.iter().for_each(|x| values.push(x));
    //             args = vmap! {
    //                 "type" => "Enum",
    //                 "name" => param.name,
    //                 "id" => param.id,
    //                 "values" => values,
    //                 "value" => param_values[0].to_owned(),
    //             }
    //         }
    //         shader_edit::ClanShaderType::Float { range } => {
    //             let from = Value::from(range[0].to_string());
    //             let to = Value::from(range[1].to_string());
    //             let step = Value::from(((range[1] - range[0]) / 20.0).to_string());
    //             args = vmap! {
    //                 "type" => "Float",
    //                 "name" => param.name,
    //                 "id" => param.id,
    //                 "from" => from,
    //                 "to" => to,
    //                 "step" => step,
    //                 "value" => 1,
    //             }
    //         }
    //         shader_edit::ClanShaderType::Int { range } => {
    //             let from = Value::from(range[0].to_string());
    //             let to = Value::from(range[1].to_string());
    //             args = vmap! {
    //                 "type" => "Int",
    //                 "name" => param.name,
    //                 "id" => param.id,
    //                 "from" => from,
    //                 "to" => to,
    //                 "step" => 1,
    //                 "value" => 1,
    //             }
    //         }
    //         shader_edit::ClanShaderType::Vector { range } => {
    //             let from = Value::from(range[0].to_string());
    //             let to = Value::from(range[1].to_string());
    //             let step = Value::from(((range[1] - range[0]) / 20.0).to_string());
    //             args = vmap! {
    //                 "type" => "Vector",
    //                 "name" => param.name,
    //                 "id" => param.id,
    //                 "from" => from,
    //                 "to" => to,
    //                 "step" => step,
    //                 "value" => 1,
    //             }
    //         }
    //     }
    //     panel
    //         .call_method("createWidget", &[args])
    //         .expect("Error while calling createWidget()");
    // }

    pub fn run(self, geng: &Geng, assets: Assets) -> Box<dyn geng::State> {
        println!("Editor run");

        let state = HeroEditorState::new(geng, assets);
        let units = state.model.units.keys().collect_vec();

        // HeroEditor::create_units_widget(panel.clone(), units);

        state
            .model
            .shaders
            .get(&state.model.selected_shader)
            .expect("Can't find selected shader")
            .parameters
            .iter()
            .for_each(|param| {
                // HeroEditor::create_widget(panel.clone(), param.clone());
            });
        Box::new(state)
    }
}

struct HeroEditorState {
    geng: Geng,
    time: Time,
    model: HeroEditorModel,
    camera: geng::Camera2d,
}

struct HeroEditorModel {
    units: HashMap<String, UnitTemplate>,
    shaders: HashMap<String, ClanShaderConfig>,
    selected_unit: String,
    selected_shader: String,
    unit_render: UnitRender,
}

impl HeroEditorModel {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let units: HashMap<String, UnitTemplate> = assets
            .units
            .map
            .iter()
            .filter(|tuple| tuple.1.tier > 0)
            .map(|tuple| (tuple.0.clone(), tuple.1.clone()))
            .collect();
        let shaders: HashMap<String, ClanShaderConfig> = assets
            .clan_shaders
            .map
            .iter()
            .map(|tuple| (tuple.0.clone(), tuple.1.clone()))
            .collect();
        let assets = Rc::new(assets);
        Self {
            unit_render: UnitRender::new(&geng, &assets),
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

impl HeroEditorState {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 10.0,
        };
        Self {
            camera,
            model: HeroEditorModel::new(geng, assets),
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

impl geng::State for HeroEditorState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.time += delta_time;
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        if self.model.selected_unit.is_empty() {
            return;
        };
        let template = self
            .model
            .units
            .get(&self.model.selected_unit)
            .expect("Can't find unit template");

        let unit = Unit::new(
            template,
            1,
            template.name.clone(),
            Faction::Player,
            Position {
                side: Faction::Player,
                x: 0,
            },
            &Statuses { map: hashmap! {} },
        );
        self.model.unit_render.draw_unit(
            &unit,
            template,
            None,
            self.time.as_f32().into(),
            &self.camera,
            framebuffer,
        );
    }
}
