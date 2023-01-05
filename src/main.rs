mod assets;
mod game;
mod logic;
mod model;
mod view;

use std::collections::*;

use assets::Assets;
use game::Game;
use geng::{prelude::*, *};
use logic::*;
use model::*;
use view::*;

type Id = i64;
type Name = String;
type Description = String;
type Time = f32;

fn main() {
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
    let state = StateManager::new();
    let model = Model {
        units: Collection::new(),
        player_team: Team {
            units: Collection::new(),
        },
        enemy_team: Team {
            units: Collection::new(),
        },
    };
    let _game = Game {
        logic,
        assets,
        view,
        state,
        model,
    };
}
