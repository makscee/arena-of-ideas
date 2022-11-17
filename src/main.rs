#![allow(dead_code, unused_mut, unused_imports, unused_variables)]
#![deny(unconditional_recursion)]

use clap::Parser;
use geng::{
    prelude::{itertools::Itertools, *},
    ui::Theme,
};
use ugli::Texture;

mod assets;
mod custom;
mod hero_edit;
mod logic;
mod model;
mod render;
mod shader_edit;
mod shop;
mod simulation;
mod tests;
mod utility;

use assets::*;
use logic::*;
use model::*;
use render::{Render, RenderModel};
use shader_edit::*;
use shop::*;
use std::cmp;
use utility::*;

use crate::simulation::walkthrough;
type Time = R32;
type Coord = i64;
type Id = i64;
type Ticks = u64;

#[derive(Clone)]
struct FrameHistory {
    time: f32,
    model: Model,
}

pub enum GameState {
    Shop,
    Battle,
}

pub struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    time: f32,
    timeline_captured: bool,
    history: Vec<FrameHistory>,
    last_frame: FrameHistory,
    logic: Logic,
    events: Events,
    render: Render,
    frame_texture: Texture,
    previous_texture: Texture,
    new_texture: Texture,
    state: GameState,
    custom: bool,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &mut Rc<Assets>,
        rounds: Vec<GameRound>,
        config: Config,
        round: usize,
        custom: bool,
        level: Option<i32>, // level of player team for custom game
    ) -> Self {
        let mut shop = Shop::new(1, &assets.units);
        let mut model = Model::new(
            config.clone(),
            assets.units.clone(),
            assets.clans.clone(),
            assets.statuses.clone(),
            round,
            rounds,
            RenderModel::new(),
            1.0,
            3,
            shop,
        );
        model.shop.refresh(&mut model.next_id, &assets.statuses);

        let mut events = Events::new(assets.options.keys_mapping.clone());
        let assets_ref = Rc::get_mut(assets).unwrap();
        let mut logic = Logic::new(model.clone(), mem::take(&mut assets_ref.sounds));

        let last_frame = FrameHistory {
            time: 0.0,
            model: logic.model.clone(),
        };
        let history = vec![last_frame.clone()];
        let render = Render::new(geng, &assets, config.fov);

        let mut game = Self {
            geng: geng.clone(),
            assets: assets.clone(),
            time: 0.0,
            history,
            render,
            timeline_captured: false,
            logic,
            events,
            last_frame,
            frame_texture: Texture::new_uninitialized(geng.ugli(), geng.window().size()),
            new_texture: Texture::new_uninitialized(geng.ugli(), geng.window().size()),
            previous_texture: Texture::new_uninitialized(geng.ugli(), geng.window().size()),
            state: GameState::Shop,
            custom,
        };
        match custom {
            true => {
                game.logic
                    .initialize_custom(&mut game.events, &config.player, level);
            }
            false => {
                game.logic.initialize(&mut game.events);
            }
        }
        game
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        if self.timeline_captured || self.logic.paused {
            return;
        }

        let index = match self
            .history
            .binary_search_by_key(&r32(geng::prelude::Float::as_f32(self.time)), |entry| {
                r32(geng::prelude::Float::as_f32(entry.time))
            }) {
            Ok(index) => index,
            Err(index) => index,
        };
        let entry = self
            .history
            .get(index)
            .unwrap_or(self.history.last().unwrap());
        let delta_time = delta_time * entry.model.time_scale * entry.model.time_modifier;
        self.time += delta_time;
        let last_frame = &self.last_frame;
        match self.state {
            GameState::Battle => {}
            GameState::Shop => {
                if self.logic.model.shop.dirty {
                    self.logic.model.shop.dirty = false;
                    self.logic.model.units = self
                        .logic
                        .model
                        .team
                        .iter()
                        .chain(self.logic.model.shop.case.iter())
                        .map(|u| u.clone())
                        .collect();
                }
            }
        }

        if self.time > self.last_frame.time {
            match self.state {
                GameState::Shop => {
                    self.history.clear();
                    self.logic
                        .model
                        .shop
                        .refresh(&mut self.logic.model.next_id, &self.logic.model.statuses);
                    if !self.custom {
                        self.logic.model.calculate_clan_members();
                    }
                }
                GameState::Battle => {
                    self.logic.update(delta_time);
                }
            }

            let new_frame = FrameHistory {
                time: self.time,
                model: self.logic.model.clone(),
            };
            self.last_frame = new_frame.clone();
            self.history.push(new_frame);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        //let _time_tracker = time_tracker.track("main draw");
        let window_size = self.geng.window().size();
        if self.frame_texture.size() != window_size {
            self.frame_texture = Texture::new_uninitialized(self.geng.ugli(), window_size);
            self.previous_texture = Texture::new_uninitialized(self.geng.ugli(), window_size);
            self.new_texture = Texture::new_uninitialized(self.geng.ugli(), window_size);
        }
        let mut game_time;
        {
            let mut framebuffer = ugli::Framebuffer::new_color(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(&mut self.frame_texture),
            );
            let framebuffer = &mut framebuffer;
            ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_WHITE), None, None);
            let index = match self
                .history
                .binary_search_by_key(&r32(geng::prelude::Float::as_f32(self.time)), |entry| {
                    r32(geng::prelude::Float::as_f32(entry.time))
                }) {
                Ok(index) => index,
                Err(index) => index,
            };
            let entry = self
                .history
                .get(index)
                .unwrap_or(self.history.last().unwrap());
            game_time = entry.time;
            self.render.draw(game_time, &entry.model, framebuffer);
        }

        let blend_shader_program = &self.assets.postfx_render.blend_shader;
        let blend_quad = blend_shader_program.get_vertices(&self.geng);
        let framebuffer_size = framebuffer.size();

        for pipe in &self.assets.postfx_render.pipes {
            let mut it = pipe.iter().peekable();
            while let Some(shader_program) = it.next() {
                {
                    let quad = shader_program.get_vertices(&self.geng);
                    let mut framebuffer = ugli::Framebuffer::new_color(
                        self.geng.ugli(),
                        ugli::ColorAttachment::Texture(&mut self.new_texture),
                    );
                    let framebuffer = &mut framebuffer;

                    ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_WHITE), None, None);
                    ugli::draw(
                        framebuffer,
                        &shader_program.program,
                        ugli::DrawMode::TriangleStrip,
                        &quad,
                        (
                            ugli::uniforms! {
                                u_time: game_time,
                                u_window_size: window_size,
                                u_previous_texture: &self.previous_texture,
                                u_frame_texture: &self.frame_texture
                            },
                            geng::camera2d_uniforms(
                                &self.render.camera,
                                framebuffer_size.map(|x| x as f32),
                            ),
                            &shader_program.parameters,
                        ),
                        ugli::DrawParameters {
                            blend_mode: Some(ugli::BlendMode::default()),
                            ..default()
                        },
                    );
                }
                mem::swap(&mut self.previous_texture, &mut self.new_texture);
                if it.peek().is_none() {
                    let mut framebuffer = ugli::Framebuffer::new_color(
                        self.geng.ugli(),
                        ugli::ColorAttachment::Texture(&mut self.new_texture),
                    );
                    let framebuffer = &mut framebuffer;
                    ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_WHITE), None, None);
                    ugli::draw(
                        framebuffer,
                        &blend_shader_program.program,
                        ugli::DrawMode::TriangleStrip,
                        &blend_quad,
                        (
                            ugli::uniforms! {
                                u_time: game_time,
                                u_window_size: window_size,
                                u_previous_texture: &self.previous_texture,
                                u_frame_texture: &self.frame_texture,
                            },
                            geng::camera2d_uniforms(
                                &self.render.camera,
                                framebuffer_size.map(|x| x as f32),
                            ),
                            &blend_shader_program.parameters,
                        ),
                        ugli::DrawParameters {
                            blend_mode: Some(ugli::BlendMode::default()),
                            ..default()
                        },
                    );
                    mem::swap(&mut self.frame_texture, &mut self.new_texture);
                }
            }
        }

        let shader_program = &self.assets.postfx_render.final_shader;
        let quad = shader_program.get_vertices(&self.geng);
        ugli::draw(
            framebuffer,
            &shader_program.program,
            ugli::DrawMode::TriangleStrip,
            &quad,
            (
                ugli::uniforms! {
                    u_frame_texture: &self.frame_texture
                },
                &shader_program.parameters,
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );
        match self.state {
            GameState::Shop => self
                .logic
                .model
                .shop
                .draw(&self.render, framebuffer, game_time),
            GameState::Battle => {}
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        match self.state {
            GameState::Shop => {
                self.logic.model.shop.handle_event(
                    &self.logic.sound_controller,
                    &self.render,
                    event,
                    &mut self.logic.model.team,
                );
            }
            GameState::Battle => match event {
                geng::Event::MouseDown { button, .. } => {
                    self.events
                        .trigger_by_key(format!("Mouse{:?}", button), &mut self.logic);
                }
                geng::Event::KeyDown { key } => {
                    self.events
                        .trigger_by_key(format!("{:?}", key), &mut self.logic);
                }
                _ => {}
            },
        }
    }
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let mut timeline = Slider::new(
            cx,
            self.time as f64,
            self.history[0].time as f64..=self.last_frame.time as f64,
            Box::new(|new_time| self.time = new_time as f32),
        );
        self.timeline_captured = timeline.sense().unwrap().is_captured();
        let mut col = geng::ui::column![];

        match self.state {
            GameState::Shop => {
                if let Some(overlay) = self.logic.model.shop.ui(
                    cx,
                    &self.logic.sound_controller,
                    &mut self.logic.model.transition,
                    &self.assets.options.clan_configs,
                    &self.logic.model.config.clans,
                ) {
                    col.push(overlay.boxed());
                }
            }
            _ => {}
        }

        col.push(
            timeline
                .constraints_override(Constraints {
                    min_size: vec2(0.0, 32.0),
                    flex: vec2(1.0, 0.0),
                })
                .align(vec2(0.5, 0.0))
                .boxed(),
        );
        col.align(vec2(0.5, 0.0)).boxed()
    }

    fn transition(&mut self) -> Option<geng::Transition> {
        if self.logic.model.transition {
            self.logic.model.transition = false;
            match self.state {
                GameState::Shop => {
                    self.state = GameState::Battle;
                    self.logic.effects.add_delay_by_id("Spawn".to_owned(), 1.0);
                    let round = self
                        .logic
                        .model
                        .rounds
                        .get(self.logic.model.round)
                        .unwrap_or_else(|| {
                            panic!("Failed to find round number: {}", self.logic.model.round)
                        })
                        .clone();
                    if !self.custom {
                        self.logic.model.calculate_clan_members();
                    }
                    self.logic.model.units.clear();
                    self.logic.init_round(round);
                    self.logic.sound_controller.start();
                }
                GameState::Battle => {
                    if self.custom {
                        return None;
                    }
                    if self.logic.effects.is_empty() {
                        self.state = GameState::Shop;
                        // Upgrade tier every 3 rounds
                        let tier = ((self.logic.model.round + 1) / 3 + 1);
                        self.logic.model.shop = Shop::new(tier, &self.assets.units);
                        self.logic
                            .model
                            .shop
                            .refresh(&mut self.logic.model.next_id, &self.logic.model.statuses);
                        self.logic.model.render_model.clear();
                        self.logic.model.round =
                            (self.logic.model.round + 1).min(self.logic.model.rounds.len() - 1);
                        self.logic.model.units.clear();
                        self.logic.model.dead_units.clear();
                        self.history = vec![self.last_frame.clone()];
                    }
                }
            }
        }
        None
    }
}

#[derive(clap::Parser)]
struct Opts {
    #[clap(long)]
    config: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    CustomGame(custom::CustomGame),
    Test,
    Shader(shader_edit::ShaderEdit),
    Simulate(simulation::Simulate),
    Walkthrough(simulation::Walkthrough),
    UpdateUnits,
    HeroEditor(hero_edit::HeroEditor),
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "Arena of Ideas".to_owned(),
        shader_prefix: Some((
            include_str!("vertex_prefix.glsl").to_owned(),
            include_str!("fragment_prefix.glsl").to_owned(),
        )),
        target_ui_resolution: Some(vec2(2560.0, 1600.0)),
        ..default()
    });
    let mut theme = Theme::dark(&geng);
    // theme.background_color = Color::WHITE;
    theme.text_color = Rgba::BLACK;
    theme.text_size = 50.0;
    theme.usable_color = Rgba::BLACK;
    geng.set_ui_theme(theme);

    // Adds restarting on R
    struct AppWrapper {
        geng: Geng,
        state_manager: geng::StateManager,
    }

    impl AppWrapper {
        fn new(geng: &Geng) -> Self {
            let opts = Opts::parse();
            let config_path = opts
                .config
                .clone()
                .unwrap_or(static_path().join("config.json"));
            let loading_screen = geng::LoadingScreen::new(
                &geng,
                geng::EmptyLoadingScreen, // TODO: change into better loading screen
                {
                    let geng = geng.clone();
                    async move {
                        let effects_path = static_path().join("effects.json");
                        Effects::load(&geng, &effects_path)
                            .await
                            .expect(&format!("Failed to load effects from {effects_path:?}"));
                        let mut assets = <Assets as geng::LoadAsset>::load(&geng, &static_path())
                            .await
                            .expect("Failed to load assets");

                        for status in assets.statuses.values_mut() {
                            let color = status.get_color(&assets.options);
                            status.status.color = color;
                        }
                        let config = <Config as geng::LoadAsset>::load(&geng, &config_path)
                            .await
                            .expect("Failed to load config");
                        (assets, config)
                    }
                },
                {
                    let geng = geng.clone();
                    move |(assets, config)| {
                        match opts.command {
                            Some(command) => match command {
                                Commands::CustomGame(custom) => {
                                    let mut assets = Rc::new(assets);
                                    return custom.run(&geng, &mut assets);
                                }
                                Commands::Test => {
                                    tests::run_tests(assets);
                                    std::process::exit(0);
                                }
                                Commands::Simulate(simulate) => {
                                    simulate.run(&geng, assets, config);
                                    std::process::exit(0);
                                }
                                Commands::Walkthrough(walkthrough) => {
                                    walkthrough.run(&geng, assets, config);
                                    std::process::exit(0);
                                }
                                Commands::Shader(shader) => {
                                    return shader.run(&geng);
                                }
                                Commands::UpdateUnits => {
                                    utility::rename_units(&geng, &static_path(), assets);
                                    std::process::exit(0);
                                }
                                Commands::HeroEditor(hero_editor) => {
                                    return hero_editor.run(&geng, assets);
                                }
                            },
                            None => (),
                        }
                        let rounds = assets.rounds.clone();
                        let mut assets = Rc::new(assets);
                        Box::new(Game::new(
                            &geng,
                            &mut assets,
                            rounds,
                            config,
                            0,
                            false,
                            None,
                        ))
                    }
                },
            );
            let mut state_manager = geng::StateManager::new(); // Needed because we don't want to transition from wrapper
            state_manager.push(Box::new(loading_screen));
            Self {
                geng: geng.clone(),
                state_manager,
            }
        }
    }

    impl geng::State for AppWrapper {
        fn update(&mut self, delta_time: f64) {
            self.state_manager.update(delta_time);
        }
        fn fixed_update(&mut self, delta_time: f64) {
            self.state_manager.fixed_update(delta_time);
        }

        fn handle_event(&mut self, event: geng::Event) {
            if let geng::Event::KeyDown { key: geng::Key::R } = event {
                *self = Self::new(&self.geng);
            }
            self.state_manager.handle_event(event);
        }

        fn transition(&mut self) -> Option<geng::Transition> {
            None // Transitions handled by inner state manager
        }

        fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
            self.state_manager.ui(cx)
        }

        fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
            self.state_manager.draw(framebuffer);
        }
    }

    geng::run(&geng, AppWrapper::new(&geng));
}
