use std::collections::VecDeque;

use geng::prelude::*;

use crate::{
    assets::{Assets, Config},
    logic::Logic,
    model::{Faction, Model, UnitTemplates},
};

#[derive(clap::Args)]
pub struct Simulate1x1 {
    player: String,
    enemy: Option<String>,
    #[clap(short, long, default_value = "1")]
    runs: usize,
    #[clap(short, long, default_value = "0.02")]
    delta_time: f32,
}

impl Simulate1x1 {
    pub fn run(self, mut assets: Assets) -> Result<(), SimulationError> {
        // Load player and enemy units
        match assets.units.contains_key(&self.player) {
            false => return Err(SimulationError::UnknownUnit(self.player)),
            true => assets.config.player = vec![self.player],
        }
        match self.enemy {
            Some(enemy) if !assets.units.contains_key(&enemy) => {
                return Err(SimulationError::UnknownUnit(enemy))
            }
            _ => (),
        }

        let mut wins = 0;
        for _ in 0..self.runs {
            let enemy = self
                .enemy
                .as_ref()
                .unwrap_or_else(|| {
                    // Select randomly
                    assets
                        .units
                        .iter()
                        .choose(&mut rand::thread_rng())
                        .expect("Could not find a random unit")
                        .0
                })
                .clone();

            let spawn_point = assets
                .config
                .spawn_points
                .iter()
                .next()
                .expect("No spawn points declared")
                .0
                .clone();
            let mut wave = HashMap::new();
            wave.insert(spawn_point, vec![enemy]);
            assets.config.waves = vec![wave];

            let simulation = Simulation::new(
                assets.units.clone(),
                assets.config.clone(),
                R32::new(self.delta_time),
            );
            let result = simulation.run();
            if result.player_won {
                wins += 1;
            }
        }

        println!("----- Simulation Results -----");
        println!(
            "Win Rate: {:.2}% ({} out of {})",
            (wins as f32 / self.runs as f32) * 100.0,
            wins,
            self.runs
        );

        Ok(())
    }
}

#[derive(Debug)]
pub enum SimulationError {
    UnknownUnit(String),
}

impl Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimulationError::UnknownUnit(name) => {
                writeln!(f, "Unknown unit: {}", name)?;
            }
        }
        Ok(())
    }
}

struct Simulation {
    config: Config,
    model: Model,
    delta_time: R32,
    // TODO: time or steps limit
}

struct SimulationResult {
    player_won: bool,
}

impl Simulation {
    pub fn new(units: UnitTemplates, config: Config, delta_time: R32) -> Self {
        Self {
            config: config.clone(),
            model: Model::new(config, units),
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
                    && self.model.config.waves.is_empty()
            {
                return SimulationResult {
                    player_won: player_alive,
                };
            }
        }
    }
}
