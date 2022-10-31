#![allow(dead_code, unused_mut, unused_imports, unused_variables)]
#![deny(unconditional_recursion)]

use clap::Parser;
use geng::{prelude::*, ui::Theme};
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
use utility::*;

use crate::simulation::walkthrough;
type Time = R32;
type Coord = i64;
type Id = i64;
type Ticks = u64;

#[derive(Clone)]
struct FrameHistory {
    time: f64,
    model: Model,
}

#[derive(PartialEq, Eq, Hash)]
pub enum GameState {
    Shop,
    Battle,
}

pub struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    time: f64,
    timeline_captured: bool,
    history: Vec<FrameHistory>,
    last_frame: FrameHistory,
    logic: Logic,
    events: Events,
    render: Render,
    frame_texture: Texture,
    previous_texture: Texture,
    new_texture: Texture,
    shop: Shop,
    state: GameState,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        config: Config,
        round: GameRound,
        custom: bool,
    ) -> Self {
        let mut model = Model::new(
            config.clone(),
            assets.units.clone(),
            assets.clans.clone(),
            assets.statuses.clone(),
            round,
            RenderModel::new(),
            1.0,
            10,
        );
        let mut events = Events::new(assets.options.keys_mapping.clone());
        let mut logic = Logic::new(model);

        let last_frame = FrameHistory {
            time: 0.0,
            model: logic.model.clone(),
        };
        let history = vec![last_frame.clone()];
        let render = Render::new(geng, &assets, &config);
        let mut shop = Shop::new(assets, render.camera.clone());
        shop.reroll(true);

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
            shop,
            state: GameState::Shop,
        };
        game.logic
            .initialize(&mut game.events, game.logic.model.round.clone());
        game
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
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
        let delta_time =
            delta_time * entry.model.time_scale as f64 * entry.model.time_modifier as f64;
        self.time += delta_time;
        let last_frame = &self.last_frame;

        if self.time > self.last_frame.time {
            if self.shop.updated && self.state == GameState::Shop {
                self.shop.updated = false;
                self.logic.model.units.clear();
                for unit in self.shop.team.iter_mut() {
                    unit.id = self.logic.model.next_id;
                    self.logic.model.next_id += 1;
                    self.logic.model.units.insert(unit.clone());
                }
            }

            if self.state == GameState::Battle {
                self.logic.update(delta_time);
            }
            if self.state == GameState::Shop && !self.shop.enabled {
                self.logic.model.transition = true;
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
            self.render
                .draw(game_time, &entry.model, &self.shop, framebuffer);
            self.shop.draw(
                &self.geng,
                &self.assets,
                game_time,
                framebuffer,
                &self.render.camera,
            );
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
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { button, .. } => {
                self.events
                    .trigger_by_key(format!("Mouse{:?}", button), &mut self.logic);
            }
            geng::Event::KeyDown { key } => {
                self.events
                    .trigger_by_key(format!("{:?}", key), &mut self.logic);
            }
            _ => {}
        }
        self.shop.handle_event(event);
    }
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let mut timeline = Slider::new(
            cx,
            self.time,
            0.0..=self.last_frame.time,
            Box::new(|new_time| self.time = new_time),
        );
        self.timeline_captured = timeline.sense().unwrap().is_captured();
        let mut col = geng::ui::column![];

        if let Some(overlay) = self.shop.ui(cx) {
            col.push(overlay.boxed());
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
        if self.last_frame.model.transition {
            match self.state {
                GameState::Shop => {
                    self.logic.model.transition = false;
                    self.state = GameState::Battle;
                    self.shop.enabled = false;
                    self.logic.model.units.clear();
                    self.logic.init_player(self.shop.team.clone());
                    self.logic.init_enemies(self.logic.model.round.clone());
                }
                GameState::Battle => {
                    if self.logic.model.visual_timer <= r32(0.0) {
                        self.logic.model.transition = false;
                        self.logic.model.render_model.clear();
                        self.state = GameState::Shop;
                        self.shop.enabled = true;
                        self.shop.updated = true;
                        self.shop.reroll(true);
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
                        let shop_config = <ShopConfig as geng::LoadAsset>::load(
                            &geng,
                            &static_path().join("shop"),
                        )
                        .await
                        .expect("Failed to load shop config");
                        (assets, config, shop_config)
                    }
                },
                {
                    let geng = geng.clone();
                    move |(assets, config, shop_config)| {
                        match opts.command {
                            Some(command) => match command {
                                Commands::CustomGame(custom) => {
                                    let assets = Rc::new(assets);
                                    return custom.run(&geng, &assets, shop_config);
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

                        let round = assets
                            .rounds
                            .get(0)
                            .unwrap_or_else(|| panic!("Failed to find round number: {}", 0))
                            .clone();
                        let assets = Rc::new(assets);

                        Box::new(Game::new(&geng, &assets, config, round, true))
                        //Box::new(shop::ShopState::new(&geng, &assets, shop_config, config))
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
