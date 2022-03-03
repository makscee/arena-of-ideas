use geng::prelude::*;

mod assets;
mod logic;
mod model;
mod render;

use assets::*;
use logic::*;
use model::*;

type Health = i32;
type Time = R32;
type Coord = R32;
type Id = i64;

pub struct Game {
    next_id: Id,
    assets: Assets,
    config: Config,
    geng: Geng,
    camera: geng::Camera2d,
    units: Collection<Unit>,
    projectiles: Collection<Projectile>,
}

impl Game {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let config: Config =
            serde_json::from_reader(std::fs::File::open(static_path().join("state.json")).unwrap())
                .unwrap();
        let mut state = Self {
            next_id: 0,
            assets,
            config: config.clone(),
            geng: geng.clone(),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 10.0,
            },
            units: Collection::new(),
            projectiles: Collection::new(),
        };
        for unit_type in &config.player {
            let template = state.assets.units.map[unit_type].clone();
            state.spawn_unit(
                &template,
                Faction::Player,
                vec2(
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                ) * Coord::new(0.01),
            );
        }
        state
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.update(Time::new(delta_time as _));
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw(framebuffer);
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
            <Assets as geng::LoadAsset>::load(&geng, static_path().to_str().unwrap()),
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
