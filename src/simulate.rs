use std::{collections::VecDeque, path::PathBuf};

use geng::prelude::*;

use crate::{
    assets::{Assets, ClanEffects, Config, GameRound, Statuses, Wave, WaveSpawn},
    logic::Logic,
    model::{Faction, Model, Unit, UnitTemplates, UnitType},
};

#[derive(clap::Args)]
pub struct Simulate {
    config_path: PathBuf,
}

#[derive(Deserialize, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
struct SimulationConfig {
    player: PlayerUnits,
    opponent: SimulationUnits,
    repeats: usize,
}

#[derive(Deserialize)]
struct PlayerUnits {
    units: Vec<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum SimulationUnits {
    Units {
        /// Each entry in the list is treated as a regular expression
        /// and will include all units, whose name satisfies it
        units: Vec<String>,
    },
    Rounds {
        from: u32,
        to: u32,
    },
}

#[derive(Debug, Deserialize)]
struct BattleConfig {
    player: Vec<UnitType>,
    opponent: Vec<UnitType>,
    repeats: usize,
}

impl SimulationConfig {
    /// Treat unit names as regular expressions and match them on `all_units`
    fn match_regex(self, all_units: &[&UnitType]) -> Self {
        Self {
            player: PlayerUnits {
                units: match_units(&self.player.units, all_units)
                    .cloned()
                    .collect(),
            },
            opponent: match self.opponent {
                SimulationUnits::Units { units } => SimulationUnits::Units {
                    units: match_units(&units, all_units).cloned().collect(),
                },
                SimulationUnits::Rounds { .. } => self.opponent,
            },
            repeats: self.repeats,
        }
    }

    fn battles(self, all_units: &[&UnitType]) -> impl Iterator<Item = BattleConfig> {
        let player = self.player.units;
        let opponent = match self.opponent {
            SimulationUnits::Units { units } => vec![units],
            SimulationUnits::Rounds { from, to } => todo!(),
        };
        opponent.into_iter().map(move |opponent| BattleConfig {
            player: player.clone(),
            opponent,
            repeats: self.repeats,
        })
    }
}

fn match_units<'a>(
    patterns: impl IntoIterator<Item = &'a String> + 'a,
    all_units: &'a [&'a UnitType],
) -> impl Iterator<Item = &'a UnitType> + 'a {
    patterns.into_iter().flat_map(move |regex| {
        let regex = regex::Regex::new(regex).expect("Failed to parse a regular expression");
        all_units
            .iter()
            .filter(move |unit| regex.is_match(unit))
            .map(|name| *name)
    })
}

#[derive(Debug, Serialize)]
struct TotalResult {
    win_rate: f64,
    games: usize,
    player: Vec<UnitType>,
}

#[derive(Debug, Serialize)]
struct BattleResult {
    win_rate: f64,
    player: Vec<UnitType>,
    opponent: Vec<UnitType>,
    games: Vec<GameResult>,
}

#[derive(Debug, Serialize)]
struct GameResult {
    winner: String,
    units_alive: Vec<UnitType>,
}

impl Simulate {
    pub fn run(self, geng: &Geng, assets: Assets, mut config: Config) {
        let config_path = static_path().join(self.config_path);
        let simulation_config = futures::executor::block_on(
            <SimulationConfig as geng::LoadAsset>::load(geng, &config_path),
        )
        .unwrap();

        let all_units = assets.units.keys().collect::<Vec<_>>();
        let simulation_config = simulation_config.match_regex(&all_units);

        let mut total_games = 0;
        let mut total_wins = 0;

        let player_units = simulation_config.player.units.clone();

        let battle_results = simulation_config
            .battles(&all_units[..])
            .map(|battle| {
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
                                        .iter()
                                        .map(|unit| WaveSpawn {
                                            r#type: unit.clone(),
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
                let mut game_wins = 0;
                let games = (1..=battle.repeats)
                    .map(|i| {
                        let result = Simulation::new(
                            config.clone(),
                            assets.clans.clone(),
                            assets.statuses.clone(),
                            round.clone(),
                            assets.units.clone(),
                            r32(0.02),
                        )
                        .run();

                        if result.player_won {
                            total_wins += 1;
                            game_wins += 1;
                        }

                        let winner = if result.player_won {
                            "player".to_string()
                        } else {
                            "opponent".to_string()
                        };
                        info!("Finished game {}/{}, winner: {winner}", i, battle.repeats);
                        GameResult {
                            winner,
                            units_alive: result
                                .units_alive
                                .into_iter()
                                .map(|unit| unit.unit_type)
                                .collect(),
                        }
                    })
                    .collect::<Vec<_>>();
                total_games += battle.repeats;
                BattleResult {
                    win_rate: if games.is_empty() {
                        0.0
                    } else {
                        game_wins as f64 / games.len() as f64
                    },
                    player: battle.player,
                    opponent: battle.opponent,
                    games,
                }
            })
            .collect::<Vec<_>>();

        let result = TotalResult {
            player: player_units,
            games: total_games,
            win_rate: if total_games == 0 {
                0.0
            } else {
                total_wins as f64 / total_games as f64
            },
        };

        let total_battles = battle_results.len();
        for (i, result) in battle_results.into_iter().enumerate() {
            info!("Battle {}/{} result: {result:#?}", i + 1, total_battles);
        }
        info!("Total result: {result:#?}");
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
    units_alive: Vec<Unit>,
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
            let finish = if self
                .model
                .units
                .iter()
                .all(|unit| !matches!(unit.faction, Faction::Player))
            {
                Some(false)
            } else if self.model.transition {
                Some(
                    self.model
                        .units
                        .iter()
                        .any(|unit| matches!(unit.faction, Faction::Player)),
                )
            } else {
                None
            };
            if let Some(player_won) = finish {
                return SimulationResult {
                    player_won,
                    units_alive: self.model.units.into_iter().collect(),
                };
            }
        }
    }
}
