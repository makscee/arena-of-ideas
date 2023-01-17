mod assets;
mod game;
mod logic;
mod model;
mod state;
mod view;

use std::collections::*;

use assets::*;
use game::Game;
use geng::{prelude::*, *};
use logic::*;
use model::*;
use state::StateManager;
use state::*;
use ugli::*;
use view::*;

type Coord = i64;
type Id = i64;
type Name = String;
type Description = String;
type Time = f32;
type Position = Vec2<f32>;

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

    let logic = Logic {
        queue: LogicQueue::new(),
        model: Model::new(),
    };
    let assets = Rc::new(
        futures::executor::block_on(<Assets as geng::LoadAsset>::load(&geng, &static_path()))
            .unwrap(),
    );

    let view = View::new(geng.clone(), assets.clone());

    // geng::run(
    //     &geng,
    //     ShaderEditState::new(&geng, assets.clone(), Rc::new(view)),
    // );

    let state = StateManager::new();
    let mut game = Game {
        geng: geng.clone(),
        logic,
        assets: assets.clone(),
        view,
        state_manager: state,
    };
    game.state_manager.push(Box::new(MainMenu {
        assets: game.assets.clone(),
        transition: false,
    }));
    geng::run(&geng, game);
}
