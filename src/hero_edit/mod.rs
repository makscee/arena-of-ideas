use std::{array, env};

use geng::prelude::itertools::Itertools;
use sciter::{dispatch_script_call, make_args, varray, vmap, Element, Value};

use crate::shader_edit::{ClanShaderParam, ShaderWidgetType};

use super::*;

#[derive(clap::Args)]
pub struct HeroEditor {}

impl HeroEditor {
    fn create_widget(panel: Element, param: ClanShaderParam) -> Value {
        let mut args = Value::new();
        match param.r#type {
            shader_edit::ShaderWidgetType::Enum => {
                let param_values = param.values.expect("No values found for Enum param");
                let mut values = Value::new();
                param_values.iter().for_each(|x| values.push(x));
                args = vmap! {
                    "type" => Value::from("Enum"),
                    "id" => param.id,
                    "values" => values,
                    "value" => param_values[0].to_owned(),
                }
            }
            shader_edit::ShaderWidgetType::Float => {
                let range = param.range.expect("No range found for Float param");
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
            shader_edit::ShaderWidgetType::Int => {}
            shader_edit::ShaderWidgetType::Vector => {}
        }
        panel
            .call_method("createWidget", &make_args!(args))
            .unwrap_or_abort()
    }

    pub fn run(self) {
        println!("Editor run");

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
        let panel = root.find_first("panel").expect("panel not found").unwrap();

        let mut param = ClanShaderParam::default();
        param.r#type = ShaderWidgetType::Enum;
        param.range = Some(vec![0.0, 3.0]);
        param.id = "u_uniform".to_string();
        param.values = Some(vec![
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
        ]);
        let value = HeroEditor::create_widget(panel, param);
    }
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
