use crate::serde_json::Value;
use std::{array, env};

use chrono::format;
use geng::prelude::*;
use geng::prelude::{itertools::Itertools, ugli::raw::RGBA};
use geng::ui::Widget;

use crate::{
    render::UnitRender,
    shader_edit::{ClanShaderParam, ClanShaderType},
};

use super::*;

mod slider;

#[derive(clap::Args)]
pub struct HeroEditor {}

impl HeroEditor {
    pub fn run(self, geng: &Geng, assets: Assets) -> Box<dyn geng::State> {
        println!("Editor run");

        let state = HeroEditorState::new(geng, assets);
        Box::new(state)
    }
}

struct HeroEditorState {
    geng: Geng,
    time: f64,
    model: HeroEditorModel,
    camera: geng::Camera2d,
}

struct HeroEditorModel {
    units: Vec<UnitTemplate>,
    shaders: Vec<ClanShaderConfig>,
    selected_unit: usize,
    selected_shader: usize,
    selected_level: usize,
    selected_clan: usize,
    unit_render: UnitRender,
    save_clicked: bool,
}

impl HeroEditorModel {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let units = assets
            .units
            .map
            .iter()
            .filter(|tuple| tuple.1.tier > 0)
            .map(|tuple| tuple.1.clone())
            .collect_vec();
        let shaders = assets
            .clan_shaders
            .values()
            .map(|x| x.clone())
            .collect_vec();
        let assets = Rc::new(assets);
        Self {
            unit_render: UnitRender::new(&geng, &assets),
            selected_unit: 0,
            selected_shader: 0,
            selected_level: 0,
            selected_clan: 0,
            units,
            shaders,
            save_clicked: false,
        }
    }
}

impl HeroEditorState {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let camera = geng::Camera2d {
            center: vec2(-1.5, 0.0),
            rotation: 0.0,
            fov: 5.0,
        };
        Self {
            camera,
            model: HeroEditorModel::new(geng, assets),
            geng: geng.clone(),
            time: 0.0,
        }
    }

    pub fn save(&self) {
        let unit = &self.model.units[self.model.selected_unit];
        let unit_str = std::fs::read_to_string(&unit.path).expect("Cant load unit file");
        let mut template: Value = serde_json::from_str(&unit_str).unwrap();
        template["clan_renders"] = serde_json::to_value(unit.clan_renders.clone()).unwrap();
        let data = serde_json::to_string_pretty(&template).expect("Failed to serialize item");
        std::fs::write(&unit.path, data)
            .expect(&format!("Cannot save unit file: {:?}", &unit.path));
    }
}

use geng::ui::*;

fn draw_slider<'a>(
    cx: &'a geng::ui::Controller,
    value: &mut f64,
    title: String,
    range: Vec<f64>,
) -> impl Widget + 'a {
    let slider_value = format!("{:.3}", value);
    let slider = slider::Slider::new(cx, *value as f64, range[0]..=range[1]);

    if let Some(change) = slider.get_change() {
        *value = change;
    }
    (
        geng::ui::ColorBox::new(Rgba::try_from("#4d9ea777").unwrap()),
        (
            Text::new(title, cx.geng().default_font(), 40.0, Rgba::BLACK)
                .padding_horizontal(16.0)
                .center()
                .background_color(Rgba::try_from("#1491d477").unwrap()),
            (
                slider
                    .padding_horizontal(16.0)
                    .padding_vertical(8.0)
                    .constraints_override(Constraints {
                        min_size: vec2(220.0, 0.0),
                        flex: vec2(0.0, 1.0),
                    }),
                Text::new(slider_value, cx.geng().default_font(), 32.0, Rgba::BLACK)
                    .padding_horizontal(16.0)
                    .padding_vertical(8.0)
                    .constraints_override(Constraints {
                        min_size: vec2(50.0, 0.0),
                        flex: vec2(0.0, 1.0),
                    }),
            )
                .row()
                .constraints_override(Constraints {
                    min_size: vec2(0.0, 60.0),
                    flex: vec2(0.0, 0.0),
                }),
        )
            .column(),
    )
        .stack()
        .uniform_padding(8.0)
}

fn draw_slider_vector<'a>(
    cx: &'a geng::ui::Controller,
    value_y: &mut f64,
    value_x: &mut f64,
    title: String,
    range: Vec<f64>,
) -> impl Widget + 'a {
    let slider_value = format!("{:.2} : {:.2}", value_x, value_y);
    let slider_x = slider::Slider::new(cx, *value_x as f64, range[0]..=range[1]);
    let slider_y = slider::Slider::new(cx, *value_y as f64, range[0]..=range[1]);
    if let Some(change) = slider_x.get_change() {
        *value_x = change;
    }
    if let Some(change) = slider_y.get_change() {
        *value_y = change;
    }
    (
        geng::ui::ColorBox::new(Rgba::try_from("#36b3c177").unwrap())
            .fixed_size(vec2(100.0, 145.0)),
        (
            title.center().fixed_size(vec2(150.0, 40.0)).center(),
            slider_value
                .center()
                .padding_horizontal(64.0)
                .padding_vertical(16.0),
            (
                slider_x.padding_vertical(8.0).fixed_size(vec2(200.0, 50.0)),
                slider_y.padding_vertical(8.0).fixed_size(vec2(200.0, 50.0)),
            )
                .column()
                .center(),
        )
            .column(),
    )
        .stack()
        .uniform_padding(16.0)
}

fn draw_button<'a>(cx: &'a geng::ui::Controller, text: String) -> geng::ui::Button {
    geng::ui::Button::new(cx, &text)
}

fn draw_selector<'a>(
    cx: &'a geng::ui::Controller,
    selected: &mut usize,
    values: &Vec<String>,
    title: String,
) -> impl Widget + 'a {
    let minus_button = geng::ui::Button::new(cx, "<");
    let plus_button = geng::ui::Button::new(cx, ">");
    if minus_button.was_clicked() {
        *selected = (*selected + values.len() - 1) % values.len();
    }
    if plus_button.was_clicked() {
        *selected = (*selected + values.len() + 1) % values.len();
    }

    (
        geng::ui::ColorBox::new(Rgba::try_from("#36b3c177").unwrap()).fixed_size(vec2(250.0, 10.0)),
        (
            title.center().fixed_size(vec2(150.0, 40.0)).center(),
            (
                minus_button,
                values[*selected]
                    .to_string()
                    .center()
                    .fixed_size(vec2(150.0, 40.0)),
                plus_button,
            )
                .row()
                .center()
                .uniform_padding(8.0),
        )
            .column()
            .boxed(),
    )
        .stack()
        .fixed_size(vec2(220.0, 110.0))
        .uniform_padding(16.0)
}

impl geng::State for HeroEditorState {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time;
        if self.model.save_clicked {
            self.save();
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_WHITE), None, None);
        let template = &self.model.units[self.model.selected_unit];

        let mut unit = Unit::new(
            &template,
            1,
            template.name.clone(),
            Faction::Player,
            Position::zero(Faction::Player),
            &Statuses { map: hashmap! {} },
        );
        unit.render.render_position.y = r32(1.0);
        unit.render.render_position.x = r32(1.0);
        (0..3).into_iter().for_each(|index| {
            self.model
                .unit_render
                .draw_unit(&unit, None, self.time, &self.camera, framebuffer);
            unit.level_up(unit.clone());
            unit.render.render_position.y -= r32(1.0);
        });
    }

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn Widget + 'a> {
        let units = self
            .model
            .units
            .iter()
            .map(|v| v.name.clone())
            .collect_vec();

        let mut widgets = vec![];
        widgets.push(
            draw_selector(
                cx,
                &mut self.model.selected_unit,
                &units,
                "Selected unit".to_string(),
            )
            .boxed(),
        );
        widgets.push(
            draw_selector(
                cx,
                &mut self.model.selected_level,
                &vec![1.to_string(), 2.to_string(), 3.to_string()],
                "Level".to_string(),
            )
            .boxed(),
        );

        let selected_unit = &self.model.units[self.model.selected_unit].clone();
        widgets.push(
            draw_selector(
                cx,
                &mut self.model.selected_clan,
                &selected_unit
                    .clans
                    .iter()
                    .map(|clan| clan.to_string())
                    .collect(),
                "Clan".to_string(),
            )
            .boxed(),
        );
        self.model.selected_shader = self
            .model
            .shaders
            .iter()
            .map(|x| x.path.clone())
            .position(|x| {
                x == selected_unit.clan_renders[self.model.selected_level][self.model.selected_clan]
                    .path
            })
            .expect("Shader not found");
        widgets.push(
            draw_selector(
                cx,
                &mut self.model.selected_shader,
                &self
                    .model
                    .shaders
                    .iter()
                    .map(|shader| shader.name.clone())
                    .collect(),
                "Shader".to_string(),
            )
            .boxed(),
        );

        let mut clan_renders = selected_unit.clan_renders[self.model.selected_level].clone();

        if clan_renders.is_empty() {
            selected_unit
                .clans
                .iter()
                .for_each(|clan| clan_renders.push(ShaderConfig::default()));
            self.model.units[self.model.selected_unit]
                .clan_renders
                .iter_mut()
                .for_each(|render| {
                    *render = clan_renders.iter().map(|render| render.clone()).collect()
                });
        };

        let mut shader_parameters = clan_renders[self.model.selected_clan].parameters.clone();

        for ele in self.model.shaders[self.model.selected_shader]
            .parameters
            .iter()
        {
            match &ele.value {
                ClanShaderType::Int { range, default } => {
                    let mut value;
                    if shader_parameters.0.contains_key(&ele.id) {
                        value = shader_parameters.0[&ele.id].clone();
                    } else {
                        value = ShaderParameter::Int(default.unwrap_or_default());
                    }
                    let mut new_value;
                    match value {
                        ShaderParameter::Int(v) => new_value = v.clone() as f64,
                        _ => panic!("Wrong parameter type"),
                    }
                    widgets.push(
                        draw_slider(
                            cx,
                            &mut new_value,
                            ele.name.to_string(),
                            range.iter().map(|x| *x as f64).collect_vec(),
                        )
                        .boxed(),
                    );
                    shader_parameters
                        .0
                        .insert(ele.id.clone(), ShaderParameter::Int(new_value as i32));
                }
                ClanShaderType::Enum { values, show_all } => {
                    let mut value;
                    if shader_parameters.0.contains_key(&ele.id) {
                        value = shader_parameters.0[&ele.id].clone();
                    } else {
                        value = ShaderParameter::Int(0);
                    }
                    let mut new_value;
                    match value {
                        ShaderParameter::Int(v) => new_value = v.clone() as usize,
                        _ => panic!("Wrong parameter type"),
                    }
                    widgets.push(
                        draw_selector(cx, &mut new_value, values, ele.name.to_string()).boxed(),
                    );
                    shader_parameters
                        .0
                        .insert(ele.id.clone(), ShaderParameter::Int(new_value as i32));
                }
                ClanShaderType::Float { range, default } => {
                    let mut value;
                    if shader_parameters.0.contains_key(&ele.id) {
                        value = shader_parameters.0[&ele.id].clone();
                    } else {
                        value = ShaderParameter::Float(default.unwrap_or_default());
                    }
                    let mut new_value;
                    match value {
                        ShaderParameter::Float(v) => new_value = v.clone() as f64,
                        _ => panic!("Wrong parameter type"),
                    }
                    widgets.push(
                        draw_slider(
                            cx,
                            &mut new_value,
                            ele.name.to_string(),
                            range.iter().map(|x| *x as f64).collect_vec(),
                        )
                        .boxed(),
                    );
                    shader_parameters
                        .0
                        .insert(ele.id.clone(), ShaderParameter::Float(new_value as f32));
                }
                ClanShaderType::Vector { range, default } => {
                    let mut value;
                    if shader_parameters.0.contains_key(&ele.id) {
                        value = shader_parameters.0[&ele.id].clone();
                    } else {
                        value = ShaderParameter::Vec2(default.unwrap_or_else(|| Vec2::ZERO));
                    }
                    let mut new_value_x;
                    let mut new_value_y;
                    match value {
                        ShaderParameter::Vec2(v) => {
                            new_value_x = v.x.clone() as f64;
                            new_value_y = v.y.clone() as f64;
                        }
                        _ => panic!("Wrong parameter type"),
                    }
                    widgets.push(
                        draw_slider_vector(
                            cx,
                            &mut new_value_x,
                            &mut new_value_y,
                            ele.name.to_string(),
                            range.iter().map(|x| *x as f64).collect_vec(),
                        )
                        .boxed(),
                    );
                    shader_parameters.0.insert(
                        ele.id.clone(),
                        ShaderParameter::Vec2(vec2(new_value_x as f32, new_value_y as f32)),
                    );
                }
            }
        }

        self.model.units[self.model.selected_unit].clan_renders[self.model.selected_level]
            [self.model.selected_clan]
            .path = self.model.shaders[self.model.selected_shader].path.clone();
        self.model.units[self.model.selected_unit].clan_renders[self.model.selected_level]
            [self.model.selected_clan]
            .parameters = shader_parameters;

        let save_button = draw_button(cx, "SAVE".to_string());
        if save_button.was_clicked() {
            self.model.save_clicked = true;
        }
        widgets.push(save_button.fixed_size(vec2(170.0, 100.0)).center().boxed());
        let mut col1 = geng::ui::column![];
        let mut col2 = geng::ui::column![];
        let widgets_len = widgets.len();
        for (index, widget) in widgets.into_iter().enumerate() {
            if index < widgets_len / 2 {
                col1.push(widget);
            } else {
                col2.push(widget);
            }
        }

        (col1, col2.fixed_size(vec2(350.0, 5000.0)))
            .row()
            .align(vec2(0.0, 1.0))
            .boxed()
    }
}
