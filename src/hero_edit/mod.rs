use std::{array, env};

use geng::prelude::*;
use geng::prelude::{itertools::Itertools, ugli::raw::RGBA};
use geng::ui::Widget;

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
    counter: usize,
    values: Vec<String>,
    slider: f64,
    index: usize,
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
            counter: 0,
            slider: 0.0,
            values: vec![
                "First".to_string(),
                "Second".to_string(),
                "Third".to_string(),
            ],
            index: 0,
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

use geng::ui::*;

fn draw_slider<'a>(
    cx: &'a geng::ui::Controller,
    value: f64,
    title: String,
    setter: Box<dyn FnMut(f64) + 'a>,
) -> impl Widget + 'a {
    let slider_value = format!("{:.1}", value);
    let slider = geng::ui::Slider::new(cx, value, 0.0..=10.0, setter).fixed_size(vec2(300.0, 1.0));
    (
        geng::ui::ColorBox::new(Color::try_from("#36b3c177").unwrap())
            .fixed_size(vec2(550.0, 45.0)),
        (
            title.center(),
            (
                slider.padding_vertical(16.0),
                slider_value
                    .center()
                    .padding_horizontal(64.0)
                    .padding_vertical(16.0),
            )
                .row()
                .center(),
        )
            .column(),
    )
        .stack()
        .uniform_padding(16.0)
}

fn draw_slider_vector<'a>(
    cx: &'a geng::ui::Controller,
    value: Vec2<f64>,
    title: String,
    setter: Vec2<Box<dyn FnMut(f64) + 'a>>,
) -> impl Widget + 'a {
    let slider_value = format!("{:.1} : {:.1}", value.x, value.y);
    let slider_x =
        geng::ui::Slider::new(cx, value.x, 0.0..=10.0, setter.x).fixed_size(vec2(300.0, 50.0));
    let slider_y =
        geng::ui::Slider::new(cx, value.y, 0.0..=10.0, setter.y).fixed_size(vec2(300.0, 50.0));
    (
        geng::ui::ColorBox::new(Color::try_from("#36b3c177").unwrap())
            .fixed_size(vec2(550.0, 145.0)),
        (
            title.center(),
            slider_value
                .center()
                .padding_horizontal(64.0)
                .padding_vertical(16.0),
            (
                slider_x.padding_vertical(8.0),
                slider_y.padding_vertical(8.0),
            )
                .column()
                .center(),
        )
            .column(),
    )
        .stack()
        .uniform_padding(16.0)
}

fn draw_selector<'a>(
    cx: &'a geng::ui::Controller,
    selected: &'a mut String,
    values: &Vec<String>,
    title: String,
) -> impl Widget + 'a {
    let minus_button = geng::ui::Button::new(cx, "<");
    let plus_button = geng::ui::Button::new(cx, ">");
    let mut index = values
        .iter()
        .position(|v| v == selected)
        .unwrap_or_default();
    if minus_button.was_clicked() {
        index = (index + values.len() - 1) % values.len();
        *selected = values[index].clone();
    }
    if plus_button.was_clicked() {
        index = (index + values.len() + 1) % values.len();
        *selected = values[index].clone();
    }

    (
        geng::ui::ColorBox::new(Color::try_from("#36b3c177").unwrap())
            .fixed_size(vec2(550.0, 25.0)),
        (
            title.center(),
            (
                minus_button,
                values[index]
                    .to_string()
                    .center()
                    .fixed_size(vec2(300.0, 80.0)),
                plus_button,
            )
                .row()
                .center()
                .uniform_padding(16.0),
        )
            .column()
            .boxed(),
    )
        .stack()
        .uniform_padding(16.0)
}

impl geng::State for HeroEditorState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.time += delta_time;
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::TRANSPARENT_WHITE), None);
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

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn Widget + 'a> {
        let units = self.model.units.keys().cloned().collect();

        let selected_shader = self.model.selected_shader.clone();
        let shader_config = self
            .model
            .shaders
            .get_mut(&selected_shader)
            .expect("Shader not found");
        let mut shader_params = &mut shader_config.parameters;

        let mut widgets = geng::ui::column![];
        widgets.push(
            draw_selector(
                cx,
                &mut self.model.selected_unit,
                &units,
                "Selected unit".to_string(),
            )
            .boxed(),
        );

        for ele in shader_params.iter_mut() {
            match &mut ele.value {
                ClanShaderType::Enum {
                    values,
                    show_all,
                    value,
                } => widgets.push(Box::new(draw_selector(
                    cx,
                    value,
                    &values,
                    ele.name.to_string(),
                ))),
                ClanShaderType::Float { range, value } => {
                    widgets.push(
                        draw_slider(
                            cx,
                            value.clone(),
                            ele.name.to_string(),
                            Box::new(|v| *value = v),
                        )
                        .boxed(),
                    );
                }
                ClanShaderType::Int { range, value } => {
                    widgets.push(
                        draw_slider(
                            cx,
                            (*value) as f64,
                            ele.name.to_string(),
                            Box::new(|v| *value = v as i64),
                        )
                        .boxed(),
                    );
                }
                ClanShaderType::Vector { range, value } => {
                    widgets.push(
                        draw_slider_vector(
                            cx,
                            *value,
                            ele.name.to_string(),
                            vec2(Box::new(|v| value.x = v), Box::new(|v| value.y = v)),
                        )
                        .boxed(),
                    );
                }
            }
        }

        // draw_slider(cx, &mut self.slider, "Slider title".to_string()),
        widgets.align(vec2(0.0, 1.0)).boxed()
    }
}
