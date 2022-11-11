use crate::serde_json::Value;
use std::path::Path;
use std::{array, env};

use chrono::format;
use geng::prelude::ugli::uniforms;
use geng::prelude::{itertools::Itertools, ugli::raw::RGBA};
use geng::ui::Widget;
use geng::{prelude::*, Draw2d};

use crate::{
    render::UnitRender,
    shader_edit::{ClanShaderParam, ClanShaderType},
};

use super::*;

mod slider;
mod vec2slider;

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
    time: f32,
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
    shift_pressed: bool,
}

impl HeroEditorModel {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let units = assets
            .units
            .map
            .iter()
            .filter(|tuple| tuple.1.tier > 0)
            .map(|tuple| tuple.1.clone())
            .sorted_by(|a, b| Ord::cmp(&a.clan_renders[0].len(), &b.clan_renders[0].len()))
            .map(|mut unit| {
                if unit.clans.len() > unit.clan_renders[0].len() {
                    let mut parameters = ShaderParameters::new();
                    parameters.0.extend(HashMap::from([(
                        "u_count".to_string(),
                        ShaderParameter::Int(1),
                    )]));
                    parameters.0.extend(HashMap::from([(
                        "u_fill".to_string(),
                        ShaderParameter::Int(1),
                    )]));
                    let mut clan_renders = vec![];
                    for i in 0..3 {
                        let mut configs = vec![];
                        for i in 0..unit.clans.len() {
                            parameters.0.insert(
                                "u_size".to_string(),
                                ShaderParameter::Float(1.0 / (i as f32 + 2.0)),
                            );
                            configs.push(ShaderConfig {
                                path: "shaders/clan_shaders/circle.glsl".to_string(),
                                parameters: parameters.clone(),
                                vertices: default(),
                                instances: default(),
                            });
                        }
                        clan_renders.push(configs);
                    }
                    unit.clan_renders = clan_renders;
                }
                unit
            })
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
            shift_pressed: false,
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
        let mut path = Path::new(&unit.path).to_owned();
        path.set_file_name(format!("{}_render.json", unit.name));
        let renders: Vec<Vec<ShaderConfig>> = unit.clan_renders.clone();
        let data = serde_json::to_string_pretty(&renders).expect("Failed to serialize item");
        std::fs::write(&path, data).expect(&format!("Cannot save unit file: {:?}", &path));
    }
}

use geng::ui::*;

fn change_rounded(change: f32, shift: bool) -> f32 {
    let mut k = 100.0;
    if shift {
        k = 10.0;
    }
    (change * k).round() / k
}

fn draw_slider<'a>(
    cx: &'a geng::ui::Controller,
    value: &mut f32,
    title: String,
    range: Vec<f32>,
    shift: bool,
) -> impl Widget + 'a {
    let slider_value = format!("{:.2}", value);
    let slider = slider::Slider::new(cx, *value, range[0]..=range[1]);
    let bg_color = match shift {
        false => Rgba::try_from("#4d9ea777").unwrap(),
        true => Rgba::try_from("#315e6377").unwrap(),
    };

    if let Some(change) = slider.get_change() {
        *value = change_rounded(change, shift);
    }
    (
        geng::ui::ColorBox::new(bg_color),
        (
            Text::new(title, cx.geng().default_font(), 40.0, Rgba::BLACK)
                .padding_horizontal(16.0)
                .center()
                .background_color(bg_color),
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
    value: &mut Vec2<f32>,
    title: String,
    range: Vec<f32>,
    shift: bool,
) -> impl Widget + 'a {
    let slider_value = format!("{:.2} : {:.2}", value.x, value.y);
    let slider = vec2slider::Vec2Slider::new(cx, *value);
    let slider_x = slider::Slider::new(cx, value.x, range[0]..=range[1]);
    let slider_y = slider::Slider::new(cx, value.y, range[0]..=range[1]);
    let bg_color = match shift {
        false => Rgba::try_from("#4d9ea777").unwrap(),
        true => Rgba::try_from("#315e6377").unwrap(),
    };
    if let Some(change) = slider.get_change() {
        *value = change.map(|x| change_rounded(x, shift));
    }
    if let Some(change) = slider_x.get_change() {
        value.x = change_rounded(change, shift);
    }
    if let Some(change) = slider_y.get_change() {
        value.y = change_rounded(change, shift);
    }
    (
        geng::ui::ColorBox::new(bg_color),
        (
            Text::new(title, cx.geng().default_font(), 40.0, Rgba::BLACK)
                .padding_horizontal(16.0)
                .center()
                .background_color(bg_color),
            (
                Text::new(slider_value, cx.geng().default_font(), 32.0, Rgba::BLACK)
                    .padding_horizontal(16.0)
                    .center(),
                slider.fixed_size(vec2(130.0, 130.0)).center(),
                slider_x.fixed_size(vec2(300.0, 25.0)).center(),
                slider_y.fixed_size(vec2(300.0, 25.0)).center(),
            )
                .column(),
        )
            .column(),
    )
        .stack()
        .uniform_padding(8.0)
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
            Text::new(title, cx.geng().default_font(), 40.0, Rgba::BLACK)
                .padding_horizontal(16.0)
                .center()
                .background_color(Rgba::try_from("#1491d477").unwrap()),
            (
                minus_button,
                Text::new(
                    values[*selected].to_string(),
                    cx.geng().default_font(),
                    35.0,
                    Rgba::BLACK,
                )
                .center()
                .fixed_size(vec2(170.0, 40.0)),
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
        self.time += delta_time as f32;
        if self.model.save_clicked {
            self.save();
            self.model.save_clicked = false;
        }
        self.model.shift_pressed = self.geng.window().is_key_pressed(geng::Key::LShift);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_WHITE), None, None);
        let template = &self.model.units[self.model.selected_unit];

        let mut unit = Unit::new(
            &template,
            1,
            Position::zero(Faction::Player),
            &Statuses { map: hashmap! {} },
        );
        unit.render.render_position.y = r32(1.0);
        unit.render.render_position.x = r32(1.0);
        let selection_pos =
            unit.render.render_position - vec2(r32(0.0), r32(self.model.selected_level as f32));
        let unit_aabb = AABB::point(selection_pos.map(|x| x.as_f32())).extend_uniform(0.5);
        draw_2d::Quad::new(unit_aabb, Rgba::try_from("#36b3c177").unwrap()).draw_2d(
            &self.geng,
            framebuffer,
            &self.camera,
        );
        (0..3).into_iter().for_each(|index| {
            self.model
                .unit_render
                .draw_unit(&unit, None, self.time, &self.camera, framebuffer);
            unit.stats.stacks += STACKS_PER_LVL;
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

        let mut col_left = geng::ui::column![];
        let mut col_right = geng::ui::column![];
        let prev_selected_unit = self.model.selected_unit;
        col_left.push(
            draw_selector(
                cx,
                &mut self.model.selected_unit,
                &units,
                "Selected unit".to_string(),
            )
            .boxed(),
        );
        if prev_selected_unit != self.model.selected_unit {
            self.model.selected_level = 0;
            self.model.selected_clan = 0;
        }
        col_left.push(
            draw_selector(
                cx,
                &mut self.model.selected_level,
                &vec![1.to_string(), 2.to_string(), 3.to_string()],
                "Level".to_string(),
            )
            .boxed(),
        );

        let selected_unit = &self.model.units[self.model.selected_unit].clone();
        col_left.push(
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
        col_left.push(
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
                        ShaderParameter::Int(v) => new_value = v.clone() as f32,
                        _ => panic!("Wrong parameter type"),
                    }
                    col_right.push(
                        draw_slider(
                            cx,
                            &mut new_value,
                            ele.name.to_string(),
                            range.iter().map(|x| *x as f32).collect_vec(),
                            self.model.shift_pressed,
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
                    col_left.push(
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
                        ShaderParameter::Float(v) => new_value = v.clone() as f32,
                        _ => panic!("Wrong parameter type"),
                    }
                    col_right.push(
                        draw_slider(
                            cx,
                            &mut new_value,
                            ele.name.to_string(),
                            range.iter().map(|x| *x as f32).collect_vec(),
                            self.model.shift_pressed,
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
                    let mut new_value;
                    match value {
                        ShaderParameter::Vec2(v) => {
                            new_value = v.clone();
                        }
                        _ => panic!("Wrong parameter type"),
                    }
                    col_right.push(
                        draw_slider_vector(
                            cx,
                            &mut new_value,
                            ele.name.to_string(),
                            range.iter().map(|x| *x).collect_vec(),
                            self.model.shift_pressed,
                        )
                        .boxed(),
                    );
                    shader_parameters
                        .0
                        .insert(ele.id.clone(), ShaderParameter::Vec2(new_value));
                }
            }
        }

        let copy_lvl_button = draw_button(cx, "COPY PREV LVL".to_string());
        let copy_clan_button = draw_button(cx, "COPY PREV CLAN".to_string());
        if copy_lvl_button.was_clicked() && self.model.selected_level > 0 {
            for i in 0..selected_unit.clans.len() {
                self.model.units[self.model.selected_unit].clan_renders
                    [self.model.selected_level][i]
                    .path = self.model.units[self.model.selected_unit].clan_renders
                    [self.model.selected_level - 1][i]
                    .path
                    .clone();
                self.model.units[self.model.selected_unit].clan_renders
                    [self.model.selected_level][i]
                    .parameters = self.model.units[self.model.selected_unit].clan_renders
                    [self.model.selected_level - 1][i]
                    .parameters
                    .clone();
            }
            self.model.selected_clan = 0;
        } else if copy_clan_button.was_clicked() && self.model.selected_clan > 0 {
            self.model.units[self.model.selected_unit].clan_renders[self.model.selected_level]
                [self.model.selected_clan]
                .path = self.model.units[self.model.selected_unit].clan_renders
                [self.model.selected_level][self.model.selected_clan - 1]
                .path
                .clone();
            self.model.units[self.model.selected_unit].clan_renders[self.model.selected_level]
                [self.model.selected_clan]
                .parameters = self.model.units[self.model.selected_unit].clan_renders
                [self.model.selected_level][self.model.selected_clan - 1]
                .parameters
                .clone();
        } else {
            self.model.units[self.model.selected_unit].clan_renders[self.model.selected_level]
                [self.model.selected_clan]
                .path = self.model.shaders[self.model.selected_shader].path.clone();
            self.model.units[self.model.selected_unit].clan_renders[self.model.selected_level]
                [self.model.selected_clan]
                .parameters = shader_parameters;
        }

        let save_button = draw_button(cx, "SAVE".to_string());
        if save_button.was_clicked() {
            self.model.save_clicked = true;
        }
        if self.model.selected_level > 0 {
            col_right.push(
                copy_lvl_button
                    .fixed_size(vec2(240.0, 40.0))
                    .uniform_padding(16.0)
                    .background_color(Rgba::try_from("#aabbff").unwrap())
                    .center()
                    .boxed(),
            )
        }
        if self.model.selected_clan > 0 {
            col_right.push(
                copy_clan_button
                    .fixed_size(vec2(240.0, 40.0))
                    .uniform_padding(16.0)
                    .background_color(Rgba::try_from("#aabbff").unwrap())
                    .center()
                    .boxed(),
            )
        }
        col_left.push(
            save_button
                .fixed_size(vec2(170.0, 80.0))
                .uniform_padding(16.0)
                .background_color(Rgba::try_from("#aabbff").unwrap())
                .center()
                .boxed(),
        );

        (col_left, col_right.fixed_size(vec2(350.0, 5000.0)))
            .row()
            .align(vec2(0.0, 1.0))
            .boxed()
    }
}
