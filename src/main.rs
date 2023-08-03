use geng::prelude::*;
use geng::ui;

mod components;
pub mod game;
mod resources;
mod systems;

use anyhow::{Error, Result};
use components::*;
use game::*;
use geng::Key::*;
use legion::query::*;
use legion::EntityStore;
use resources::{Resources, *};
use std::path::PathBuf;
use systems::*;

type Time = f32;

fn setup_geng() -> Geng {
    geng::setup_panic_handler();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "Arena of Ideas".to_owned(),
        antialias: true,
        shader_prefix: Some((
            include_str!("vertex_prefix.glsl").to_owned(),
            include_str!("fragment_prefix.glsl").to_owned(),
        )),
        target_ui_resolution: Some(vec2(1920.0, 1080.0)),
        window_size: Some(vec2(1920, 1080)),
        ..default()
    });
    geng
}

fn static_path() -> PathBuf {
    run_dir().join("static")
}
fn ratings_path() -> PathBuf {
    run_dir().join("ratings")
}
fn ts_nano() -> i64 {
    chrono::prelude::Utc::now().timestamp_nanos()
}
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}
fn mix_vec(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
    vec2(mix(a.x, b.x, t), mix(a.y, b.y, t))
}
fn new_entity() -> legion::Entity {
    legion::world::Allocate::new().next().unwrap()
}
fn options_color(key: &str) -> Rgba<f32> {
    OPTIONS_COLORS.with(|map| {
        map.borrow()
            .get(key)
            .expect(&format!("Color Key \"{key}\" not found in options.json"))
            .clone()
    })
}
fn global_time() -> Time {
    GLOBAL_TIME.with(|value| *value.borrow())
}
fn main() {
    let timer = Instant::now();
    logger::init();

    let options = Options::do_load();
    let mut world = legion::World::default();
    let mut resources = Resources::new(options);

    let mut watcher = FileWatcherSystem::new();
    resources.load(&mut watcher);
    let geng = setup_geng();
    resources.load_geng(&mut watcher, &geng);
    Game::init_world(&mut resources, &mut world);

    let mut theme = geng.ui_theme();
    theme.font = resources.fonts.get_font(0);
    theme.hover_color = Rgba::BLACK;
    geng.set_ui_theme(theme);
    if resources.options.rate_heroes {
        // RatingSystem::simulate_walkthrough(&mut world, &mut resources);
        RatingSystem::calculate_hero_ratings(&mut world, &mut resources);
    } else if resources.options.generate_ladder {
        panic!();
        // RatingSystem::simulate_enemy_ratings_calculation(&mut world, &mut resources);
        // RatingSystem::generate_hero_ladder(&mut world, &mut resources);
    } else {
        let game = Game::new(world, resources, watcher);
        debug!("Game load in: {:?}", timer.elapsed());
        geng.clone().run(game);
    }
}
