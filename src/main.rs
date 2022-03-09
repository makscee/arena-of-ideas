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
    next_id: Id,
    assets: Assets,
    geng: Geng,
    camera: geng::Camera2d,
    delta_time: Time,
    model: Model,
    effects: Vec<QueuedEffect<Effect>>,
    pressed_keys: Vec<Key>,
    render: Render,
}

impl Game {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let mut game = Self {
            next_id: 0,
            assets,
            geng: geng.clone(),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 10.0,
            },
            delta_time: Time::new(0.0),
            model: Model::new(),
            effects: Vec::new(),
            pressed_keys: Vec::new(),
            render: Render::new(),
        };
        for unit_type in &game.assets.config.player.clone() {
            game.spawn_unit(unit_type, Faction::Player, Vec2::ZERO);
        }
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
