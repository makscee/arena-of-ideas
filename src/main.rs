mod assets;
mod game;
mod logic;
mod model;
mod state;
mod view;

use assets::Assets;
use game::Game;
use geng::{
    net::Message,
    prelude::{itertools::Itertools, *},
    ui::Theme,
};
use logic::Logic;
use model::Model;
use state::StateManager;
use view::View;

type Id = i64;
type Name = String;
type Description = String;
type Time = f32;

fn main() {
    let _game = Game {
        logic: todo!(),
        assets: todo!(),
        view: todo!(),
        state: todo!(),
    };
}
