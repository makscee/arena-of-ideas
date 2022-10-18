use std::{array, env};

use geng::prelude::itertools::Itertools;
use sciter::{dispatch_script_call, make_args, varray, vmap, Element, Value};

use crate::shader_edit::{ClanShaderParam, ClanShaderType};

use super::*;

#[derive(clap::Args)]
pub struct HeroEditor {}

impl HeroEditor {
    fn create_widget(panel: Element, param: ClanShaderParam) -> Value {
        let mut args;
        match param.value {
            shader_edit::ClanShaderType::Enum { values, show_all } => {
                let param_values = values;
                let mut values = Value::new();
                param_values.iter().for_each(|x| values.push(x));
                args = vmap! {
                    "type" => Value::from("Enum"),
                    "id" => param.id,
                    "values" => values,
                    "value" => param_values[0].to_owned(),
                }
            }
            shader_edit::ClanShaderType::Float { range } => {
                let from = Value::from(range[0].to_string());
                let to = Value::from(range[1].to_string());
                let step = Value::from(((range[1] - range[0]) / 20.0).to_string());
                args = vmap! {
                    "type" => "Float",
                    "id" => param.id,
                    "from" => from,
                    "to" => to,
                    "step" => step,
                    "value" => 1,
                }
            }
            shader_edit::ClanShaderType::Int { range } => {
                let from = Value::from(range[0].to_string());
                let to = Value::from(range[1].to_string());
                args = vmap! {
                    "type" => "Float",
                    "id" => param.id,
                    "from" => from,
                    "to" => to,
                    "step" => 1,
                    "value" => 1,
                }
            }
            shader_edit::ClanShaderType::Vector { range } => {
                let from = Value::from(range[0].to_string());
                let to = Value::from(range[1].to_string());
                let step = Value::from(((range[1] - range[0]) / 20.0).to_string());
                args = vmap! {
                    "type" => "Float",
                    "id" => param.id,
                    "from" => from,
                    "to" => to,
                    "step" => step,
                    "value" => 1,
                }
            }
        }
        panel
            .call_method("createWidget", &make_args!(args))
            .unwrap_or_abort()
    }

    pub fn run(self, geng: &Geng, assets: Assets) -> Box<dyn geng::State> {
        println!("Editor run");
        let state = HeroEditorState::new(geng, assets);

        sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8
                | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8
                | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8
                | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8,
        ))
        .unwrap();
        sciter::set_options(sciter::RuntimeOptions::DebugMode(true)).unwrap();

        let dir = env::current_dir().unwrap().as_path().display().to_string();
        let filename = format!("{}/{}", dir, "resources/index.htm");
        println!("Full filename with path of index.htm: {}", filename);

        let mut frame = sciter::Window::new();
        frame
            .set_options(sciter::window::Options::DebugMode(true))
            .unwrap();
        frame.event_handler(Handler);
        frame.load_file(&filename);
        let hwnd = frame.get_hwnd();
        frame.run_app();

        use sciter::{Element, Value};

        let root = Element::from_window(hwnd).unwrap();

        state
            .model
            .shaders
            .get(&state.model.selected_shader)
            .expect("Can't find selected shader")
            .parameters
            .iter()
            .for_each(|param| {
                let panel = root.find_first("panel").expect("panel not found").unwrap();
                HeroEditor::create_widget(panel, param.clone());
            });
        Box::new(state)
    }
}

struct HeroEditorState {
    geng: Geng,
    time: Time,
    model: HeroEditorModel,
}

struct HeroEditorModel {
    units: HashMap<String, UnitTemplate>,
    shaders: HashMap<String, ClanShaderConfig>,
    selected_unit: String,
    selected_shader: String,
}

impl HeroEditorModel {
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

impl HeroEditorState {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        Self {
            model: HeroEditorModel::new(assets),
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

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {}
}

struct Handler;

impl Handler {
    fn update_uniform(&self, id: String, value: String) {
        debug!("id={}, value={}", id, value);
    }

    fn save_uniforms(&self) {
        debug!("Save uniforms");
    }
}

impl sciter::EventHandler for Handler {
    dispatch_script_call! {
      fn update_uniform(String, String);
      fn save_uniforms();
    }
}
