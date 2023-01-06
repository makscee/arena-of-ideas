mod assets;
mod game;
mod logic;
mod model;
mod state;
mod view;

use std::collections::*;

use assets::Assets;
use game::Game;
use geng::{prelude::*, *};
use logic::*;
use model::*;
use state::*;
use ugli::*;
use view::*;

type Id = i64;
type Name = String;
type Description = String;
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

    let logic = Logic {
        queue: LogicQueue {
            nodes: VecDeque::new(),
        },
    };
    let assets = Assets {
        units: vec![],
        clans: vec![],
        rounds: vec![],
    };
    let camera = geng::Camera2d {
        center: vec2(0.0, 0.0),
        rotation: 0.0,
        fov: 5.0,
    };
    let view = View {
        queue: VisualQueue {
            nodes: VecDeque::new(),
            persistent_nodes: vec![],
        },
        camera,
    };
    let model = Model {
        units: Collection::new(),
        player_team: Team {
            units: Collection::new(),
        },
        enemy_team: Team {
            units: Collection::new(),
        },
    };

    geng::setup_panic_handler();
    let geng = setup_geng();

    let state = StateManager::new();
    let mut game = Game {
        geng: geng.clone(),
        logic,
        assets,
        view,
        state,
        model,
    };
    game.state.push(Box::new(MainMenu {
        model: Rc::new(game.model),
        view: Rc::new(game.view),
        logic: Rc::new(game.logic),
        transition: false,
    }));
    geng::run(&geng, game.state);
}
