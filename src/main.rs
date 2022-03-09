#![allow(dead_code, unused_mut, unused_imports, unused_variables)]
#![deny(unconditional_recursion)]

use geng::prelude::*;

mod assets;
mod logic;
mod model;
mod render;

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
    camera: geng::Camera2d,
    model: Model,
    pressed_keys: Vec<Key>,
    render: Render,
}

impl Game {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let mut game = Self {
            geng: geng.clone(),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 10.0,
            },
            model: Model::new(assets.config.clone(), assets.units.clone()),
            render: Render::new(),
            pressed_keys: Vec::new(),
            assets,
        };
        Logic::initialize(&mut game.model, &game.assets.config);
        game
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
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

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new("Arena of Ideas");
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            <Assets as geng::LoadAsset>::load(&geng, &static_path()),
            {
                let geng = geng.clone();
                move |assets| {
                    let assets = assets.expect("Failed to load assets");
                    Game::new(&geng, assets)
                }
            },
        ),
    );
}
