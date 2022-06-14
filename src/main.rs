#![allow(dead_code, unused_mut, unused_imports, unused_variables)]
#![deny(unconditional_recursion)]

use clap::Parser;
use geng::prelude::*;

mod assets;
mod custom;
mod logic;
mod model;
mod render;
mod shop;
mod simulate;
mod tests;

use assets::*;
use logic::*;
use model::*;
use render::{Render, RenderModel};
use shop::*;

type Health = R32;
type Time = R32;
type Coord = R32;
type Id = i64;

#[derive(Clone)]
struct HistoryEntry {
    time: f32,
    model: Model,
    render: RenderModel,
}

pub struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    time: f32,
    timeline_captured: bool,
    history: Vec<HistoryEntry>,
    pressed_keys: Vec<Key>,
    render: Render,
    shop: Shop,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        config: Config,
        shop: Shop,
        round: GameRound,
    ) -> Self {
        let mut model = Model::new(
            config.clone(),
            assets.units.clone(),
            assets.clans.clone(),
            assets.statuses.clone(),
            round,
        );
        Logic::initialize(&mut model, &config);
        let mut game = Self {
            geng: geng.clone(),
            assets: assets.clone(),
            time: 0.0,
            history: vec![HistoryEntry {
                time: 0.0,
                model,
                render: RenderModel::new(),
            }],
            render: Render::new(geng, assets, config),
            pressed_keys: Vec::new(),
            timeline_captured: false,
            shop,
        };
        game
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        if self.timeline_captured {
            return;
        }
        self.time += delta_time as f32;
        let last_entry = self.history.last().unwrap();
        if self.time > last_entry.time {
            let delta_time = self.time - last_entry.time;
            let mut new_entry = last_entry.clone();
            new_entry.model.update(
                mem::take(&mut self.pressed_keys),
                Time::new(delta_time),
                Some(&mut new_entry.render),
            );
            new_entry.render.update(delta_time);
            new_entry.time = self.time;
            self.history.push(new_entry);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let index = match self
            .history
            .binary_search_by_key(&r32(self.time), |entry| r32(entry.time))
        {
            Ok(index) => index,
            Err(index) => index,
        };
        let entry = self
            .history
            .get(index)
            .unwrap_or(self.history.last().unwrap());
        self.render
            .draw(entry.time, &entry.model, &entry.render, framebuffer);
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { button, .. } => {
                self.pressed_keys.push(format!("Mouse{:?}", button));
            }
            _ => {}
        }
    }
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let mut timeline = Slider::new(
            cx,
            self.time as f64,
            0.0..=self.history.last().unwrap().time as f64,
            Box::new(|new_time| self.time = new_time as f32),
        );
        self.timeline_captured = timeline.sense().unwrap().is_captured();
        Box::new(
            timeline
                .constraints_override(Constraints {
                    min_size: vec2(0.0, 32.0),
                    flex: vec2(1.0, 0.0),
                })
                .align(vec2(0.5, 0.0)),
        )
    }
    fn transition(&mut self) -> Option<geng::Transition> {
        self.history
            .last()
            .map(|entry| (&entry.model, entry.model.transition))
            .and_then(|(model, transition)| match transition {
                false => None,
                true => {
                    let shop_state = shop::ShopState::load(
                        &self.geng,
                        &self.assets,
                        self.shop.take(),
                        model.config.clone(),
                    );
                    Some(geng::Transition::Switch(Box::new(shop_state)))
                }
            })
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
    Simulate1x1(simulate::Simulate1x1),
}

fn main() {
    let opts = Opts::parse();

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
    let config_path = opts
        .config
        .clone()
        .unwrap_or(static_path().join("config.json"));
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            {
                let geng = geng.clone();
                async move {
                    let path = static_path().join("effects.json");
                    Effects::load(&geng, &path)
                        .await
                        .expect(&format!("Failed to load effects from {path:?}"));
                    let assets = <Assets as geng::LoadAsset>::load(&geng, &static_path())
                        .await
                        .expect("Failed to load assets");
                    let config = <Config as geng::LoadAsset>::load(&geng, &config_path)
                        .await
                        .expect("Failed to load config");
                    let shop_config =
                        <ShopConfig as geng::LoadAsset>::load(&geng, &static_path().join("shop"))
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
                            Commands::Simulate1x1(simulate) => {
                                simulate.run(assets, config).unwrap();
                                std::process::exit(0);
                            }
                        },
                        None => (),
                    }

                    let assets = Rc::new(assets);
                    Box::new(shop::ShopState::new(&geng, &assets, shop_config, config))
                }
            },
        ),
    );
}
