#![allow(dead_code, unused_mut, unused_imports, unused_variables)]
#![deny(unconditional_recursion)]

use clap::Parser;
use geng::prelude::*;

mod assets;
mod logic;
mod model;
mod render;
mod simulate;
mod tests;

use assets::*;
use logic::*;
use model::*;
use render::Render;

type Health = R32;
type Time = R32;
type Coord = R32;
type Id = i64;

pub struct Game {
    assets: Assets,
    geng: Geng,
    time: f32,
    camera: geng::Camera2d,
    model: Model,
    pressed_keys: Vec<Key>,
    render: Render,
}

impl Game {
    pub fn new(geng: &Geng, assets: Assets, config: Config) -> Self {
        let mut game = Self {
            geng: geng.clone(),
            time: 0.0,
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 20.0,
            },
            model: Model::new(config.clone(), assets.units.clone()),
            render: Render::new(),
            pressed_keys: Vec::new(),
            assets,
        };
        Logic::initialize(&mut game.model, &config);
        game
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time as f32;
        self.render.update(delta_time as _);
        self.update(Time::new(delta_time as _));
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw(framebuffer);
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { button, .. } => {
                self.pressed_keys.push(format!("Mouse{:?}", button));
            }
            _ => {}
        }
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
    Test,
    Simulate1x1(simulate::Simulate1x1),
}

fn main() {
    let opts = Opts::parse();

    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new("Arena of Ideas");
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
                    let assets = <Assets as geng::LoadAsset>::load(&geng, &static_path())
                        .await
                        .expect("Failed to load assets");
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
                            Commands::Simulate1x1(simulate) => {
                                simulate.run(assets, config).unwrap();
                                std::process::exit(0);
                            }
                            Commands::Test => {
                                tests::run_tests(assets);
                                std::process::exit(0);
                            }
                        },
                        None => (),
                    }

                    Game::new(&geng, assets, config)
                }
            },
        ),
    );
}
