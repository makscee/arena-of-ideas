use std::{collections::VecDeque, path::PathBuf};

use geng::prelude::*;

use crate::{
    assets::{Assets, ClanEffects, Config, GameRound, Statuses, Wave, WaveSpawn},
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
    Units { units: Vec<String> },
    Rounds { from: u32, to: u32 },
}

#[derive(Debug, Deserialize)]
struct BattleConfig {
    player: Vec<UnitType>,
    opponent: Vec<UnitType>,
    repeats: usize,
}

impl SimulationConfig {
    fn battles(self) -> impl Iterator<Item = BattleConfig> {
        let player = match self.player {
            SimulationUnits::Units { units } => vec![units],
            SimulationUnits::Rounds { from, to } => todo!(),
        };
        let opponent = match self.opponent {
            SimulationUnits::Units { units } => vec![units],
            SimulationUnits::Rounds { from, to } => todo!(),
        };
        player.into_iter().flat_map(move |player| {
            opponent
                .clone()
                .into_iter()
                .map(move |opponent| BattleConfig {
                    player: player.clone(),
                    opponent,
                    repeats: self.repeats,
                })
        })
    }
}

impl Simulate {
    pub fn run(self, geng: &Geng, assets: Assets, mut config: Config) {
        let config_path = static_path().join(self.config_path);
        let simulation_config = futures::executor::block_on(
            <SimulationConfig as geng::LoadAsset>::load(geng, &config_path),
        )
        .unwrap();

        for battle in simulation_config.battles() {
            info!("Starting the battle: {battle:?}");

            let round = GameRound {
                statuses: vec![],
                waves: {
                    let mut waves = VecDeque::new();
                    waves.push_back(Wave {
                        start_delay: R32::ZERO,
                        between_delay: R32::ZERO,
                        wait_clear: false,
                        statuses: vec![],
                        spawns: {
                            let spawn_point = config
                                .spawn_points
                                .keys()
                                .next()
                                .expect("Expected at least one spawn point")
                                .clone();
                            [(
                                spawn_point,
                                battle
                                    .opponent
                                    .into_iter()
                                    .map(|unit| WaveSpawn {
                                        r#type: unit,
                                        count: 1,
                                    })
                                    .collect(),
                            )]
                            .into_iter()
                            .collect()
                        },
                    });
                    waves
                },
            };
            for i in 1..=battle.repeats {
                let result = Simulation::new(
                    config.clone(),
                    assets.clans.clone(),
                    assets.statuses.clone(),
                    round.clone(),
                    assets.units.clone(),
                    r32(0.02),
                )
                .run();

                info!(
                    "Finished battle {}/{}, result: {result:?}",
                    i, battle.repeats
                );
            }
        }
    }
}

struct Simulation {
    config: Config,
    model: Model,
    delta_time: R32,
    // TODO: time or steps limit
}

#[derive(Debug)]
struct SimulationResult {
    player_won: bool,
}

impl Simulation {
    pub fn new(
        config: Config,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: GameRound,
        units_templates: UnitTemplates,
        delta_time: R32,
    ) -> Self {
        Self {
            config: config.clone(),
            model: Model::new(config, units_templates, clan_effects, statuses, round),
            delta_time,
        }
    }

    pub fn run(mut self) -> SimulationResult {
        Logic::initialize(&mut self.model, &self.config);

        loop {
            self.model.update(vec![], self.delta_time, None);
            if self
                .model
                .units
                .iter()
                .all(|unit| !matches!(unit.faction, Faction::Player))
            {
                return SimulationResult { player_won: false };
            }
            if self.model.transition {
                let player_won = self
                    .model
                    .units
                    .iter()
                    .any(|unit| matches!(unit.faction, Faction::Player));
                return SimulationResult { player_won };
            }
        }
    }
}
