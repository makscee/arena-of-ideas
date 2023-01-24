use geng::{prelude::*, *};

mod components;
mod game;
mod resources;
mod systems;

use components::*;
use game::*;
use legion::query::*;
use resources::Resources;
use std::path::PathBuf;
use systems::*;

type Time = f32;

fn setup_geng() -> Geng {
    geng::setup_panic_handler();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "Arena of Ideas".to_owned(),
        shader_prefix: Some((
            include_str!("vertex_prefix.glsl").to_owned(),
            include_str!("fragment_prefix.glsl").to_owned(),
        )),
        target_ui_resolution: Some(vec2(1920.0, 1080.0)),
        ..default()
    });
    geng
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = setup_geng();
    let mut world = legion::World::default();

    world.push((GameState::MainMenu,));

    //push unit
    world.push((
        Position(Vec2::ZERO),
        Shader {
            path: PathBuf::try_from("system/unit.glsl").unwrap(),
            parameters: ShaderParameters::new(),
            layer: ShaderLayer::Unit,
            order: 0,
        },
    ));

    let resources = Resources::new(&geng);
    let game = Game::new(world, resources);
    geng::run(&geng, game);
}
