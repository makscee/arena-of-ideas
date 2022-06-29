use std::{collections::VecDeque, path::PathBuf};

use geng::prelude::*;

use crate::{
    assets::{Assets, Config},
    logic::Logic,
    model::{Faction, Model, UnitTemplates, UnitType},
};

#[derive(clap::Args)]
pub struct Simulate {
    config_path: PathBuf,
}

#[derive(Deserialize, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
struct SimulationConfig {
    player: SimulationUnits,
    opponent: SimulationUnits,
    repeats: usize,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum SimulationUnits {
    Units { units: Vec<UnitType> },
    Rounds { from: u32, to: u32 },
}

impl Simulate {
    pub fn run(self, geng: &Geng, assets: Assets, mut config: Config) {
        let config_path = static_path().join(self.config_path);
        let config = futures::executor::block_on(<SimulationConfig as geng::LoadAsset>::load(
            geng,
            &config_path,
        ))
        .unwrap();
        todo!()
    }
}

struct SimulationState {
    config: Config,
    model: Model,
    delta_time: R32,
    // TODO: time or steps limit
}

struct SimulationResult {
    player_won: bool,
}

impl SimulationState {
    pub fn new(units: UnitTemplates, config: Config, delta_time: R32) -> Self {
        Self {
            config: config.clone(),
            model: Model::new(config, units, todo!(), todo!(), todo!()),
            delta_time,
        }
    }

    pub fn run(mut self) -> SimulationResult {
        Logic::initialize(&mut self.model, &self.config);

        loop {
            let mut logic = Logic {
                model: &mut self.model,
                delta_time: self.delta_time,
                effects: VecDeque::new(),
                pressed_keys: Vec::new(),
                render: None,
            };
            logic.process();

            let mut player_alive = false;
            let mut enemies_alive = false;
            for unit in &self.model.units {
                match unit.faction {
                    Faction::Player => player_alive = true,
                    Faction::Enemy => enemies_alive = true,
                }
                if player_alive && enemies_alive {
                    break;
                }
            }

            if !player_alive
                || !enemies_alive
                    && self.model.spawning_units.is_empty()
                    && self.model.time_bombs.is_empty()
            // && self.model.config.waves.is_empty() // TODO: fix
            {
                return SimulationResult {
                    player_won: player_alive,
                };
            }
        }
    }
}
