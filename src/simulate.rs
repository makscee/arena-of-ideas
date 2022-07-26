use std::{collections::VecDeque, path::PathBuf, time::Instant};

use geng::prelude::*;

use crate::{
    assets::{self, Assets, ClanEffects, Config, GameRound, KeyMapping, Statuses},
    logic::{Events, Logic},
    model::{Faction, Model, Unit, UnitTemplates, UnitType},
    render::RenderModel,
};

#[derive(clap::Args)]
pub struct Simulate {
    config_path: PathBuf,
}

#[derive(Deserialize, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
struct SimulationConfig {
    player: Vec<RegexUnit>,
    opponent: SimulationUnits,
    repeats: usize,
}

type RegexUnit = String;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum SimulationUnits {
    Units { units: Vec<RegexUnit> },
    Rounds { from: usize, to: usize },
}

#[derive(Debug, Deserialize)]
struct BattleConfig {
    player: Vec<UnitType>,
    round: GameRound,
    repeats: usize,
}

impl SimulationConfig {
    fn battles(
        &self,
        rounds: &[GameRound],
        all_units: &[&UnitType],
    ) -> impl Iterator<Item = BattleConfig> + '_ {
        let battles: Vec<BattleConfig> = vec![];
        let mut player_variants = vec![];
        player_variants = self.match_units(all_units, &self.player, 0, player_variants);

        let battle_rounds = match &self.opponent {
            SimulationUnits::Units { units } => {
                let mut unit_vars = vec![];
                unit_vars = self.match_units(all_units, &units, 0, unit_vars);
                let mut game_rounds = vec![];
                for variant in unit_vars {
                    game_rounds.push(GameRound {
                        statuses: vec![],
                        enemies: variant.to_vec(),
                    });
                }
                game_rounds
            }
            SimulationUnits::Rounds { from, to } => {
                rounds.iter().take(*to).skip(from - 1).cloned().collect()
            }
        };

        player_variants.into_iter().flat_map(move |player| {
            let mut rounds = vec![];
            for round in &battle_rounds {
                rounds.push(BattleConfig {
                    player: player.clone(),
                    round: round.clone(),
                    repeats: self.repeats,
                });
            }
            rounds
        })
    }

    fn match_units(
        &self,
        all_units: &[&UnitType],
        units: &Vec<RegexUnit>,
        index: usize,
        result: Vec<Vec<UnitType>>,
    ) -> Vec<Vec<UnitType>> {
        let mut cloned = result.clone();
        if index == units.len() {
            return cloned;
        }

        if cloned.is_empty() {
            cloned.push(vec![]);
        }

        let regex_units = self.to_units(units[index].clone(), all_units);
        let mut regex_peek = regex_units.into_iter().peekable();
        while let Some(unit) = regex_peek.next() {
            let mut last_index = cloned.len() - 1;
            cloned[last_index].push(unit);
            cloned = self.match_units(all_units, units, index + 1, cloned);
            last_index = cloned.len() - 1;
            if regex_peek.peek().is_some() {
                //copy last line and truncate unnessesary elements
                let mut copied_line = cloned[last_index].clone();
                copied_line.truncate(index);
                cloned.push(copied_line);
            }
        }
        cloned.clone()
    }

    fn to_units(&self, unit: RegexUnit, all_units: &[&UnitType]) -> Vec<UnitType> {
        let regex = regex::Regex::new(&unit).expect("Failed to parse a regular expression");
        all_units
            .iter()
            .filter(move |unit| regex.is_match(unit))
            .map(|name| *name)
            .cloned()
            .collect()
    }
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
    round: GameRound,
    games: Vec<GameResult>,
}

#[derive(Debug, Serialize)]
struct GameResult {
    winner: String,
    units_alive: Vec<UnitType>,
}

impl Simulate {
    pub fn run(self, geng: &Geng, assets: Assets, mut config: Config) {
        let start = Instant::now();
        let config_path = static_path().join(self.config_path);
        let simulation_config = futures::executor::block_on(
            <SimulationConfig as geng::LoadAsset>::load(geng, &config_path),
        )
        .unwrap();
        info!("Starting simulation");

        let all_units = assets.units.keys().collect::<Vec<_>>();

        let mut total_games = 0;
        let mut total_wins = 0;

        //let player_units = simulation_config.player.clone();
        let mut total_results = vec![];
        let mut last_player: Option<Vec<UnitType>> = None;
        let battles: Vec<BattleConfig> = simulation_config
            .battles(&assets.rounds, &all_units)
            .collect();
        let battle_results = battles
            .into_iter()
            .map(|battle| {
                info!("Starting the battle: {battle:?}");
                let mut game_wins = 0;
                if let Some(last_player) = &last_player {
                    if battle.player != last_player.clone() {
                        let result = TotalResult {
                            player: last_player.to_vec(),
                            games: total_games,
                            win_rate: if total_games == 0 {
                                0.0
                            } else {
                                total_wins as f64 / total_games as f64
                            },
                        };
                        total_results.push(result);
                        total_games = 0;
                        total_wins = 0;
                    }
                }
                last_player = Some(battle.player.clone());
                let games = (1..=battle.repeats)
                    .map(|i| {
                        let result = Simulation::new(
                            Config {
                                player: battle.player.clone(),
                                ..config.clone()
                            },
                            assets.clans.clone(),
                            assets.statuses.clone(),
                            battle.round.clone(),
                            assets.units.clone(),
                            0.02 as f64,
                            assets.options.keys_mapping.clone(),
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
                    round: battle.round,
                    games,
                }
            })
            .collect::<Vec<_>>();

        // push last result
        if let Some(last_player) = &last_player {
            let result = TotalResult {
                player: last_player.to_vec(),
                games: total_games,
                win_rate: if total_games == 0 {
                    0.0
                } else {
                    total_wins as f64 / total_games as f64
                },
            };
            total_results.push(result);
        }
        info!("Simulation ended: {:?}", start.elapsed());
        info!("Gathering results");
        let total_battles = battle_results.len();
        let result_path = PathBuf::new().join("simulation_result");
        let battles_path = result_path.join("battles");

        // Create directories
        match std::fs::create_dir_all(&battles_path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }

        // Write results
        write_to(result_path.join("total.json"), &total_results).expect("Failed to write results");
        for (i, result) in battle_results.iter().enumerate() {
            let path = battles_path.join(format!("battle_{}.json", i + 1));
            write_to(path, result).expect("Failed to write results");
        }
        info!("Results saved: {:?}", start.elapsed());
    }
}

fn write_to<T: Serialize>(path: impl AsRef<std::path::Path>, item: &T) -> std::io::Result<()> {
    let path = path.as_ref();
    let file = std::fs::File::create(path).expect(&format!("Failed to create {path:?}"));
    let data = serde_json::to_string_pretty(item).expect("Failed to serialize item");
    std::fs::write(path, data)?;
    Ok(())
}

struct Simulation {
    config: Config,
    key_mappings: Vec<KeyMapping>,
    model: Model,
    delta_time: f64,
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
        delta_time: f64,
        key_mappings: Vec<KeyMapping>,
    ) -> Self {
        Self {
            config: config.clone(),
            key_mappings,
            model: Model::new(
                config,
                units_templates,
                clan_effects,
                statuses,
                round,
                RenderModel::new(),
            ),
            delta_time,
        }
    }

    pub fn run(mut self) -> SimulationResult {
        let mut logic = Logic::new(self.model.clone());
        let mut events = Events::new(self.key_mappings);
        logic.initialize(
            &mut events,
            self.config.player.clone(),
            self.model.round.clone(),
        );

        loop {
            logic.update(self.delta_time);
            let model = &logic.model;
            if model.transition || model.current_tick.tick_num > 100 {
                let player_won = model
                    .units
                    .iter()
                    .all(|unit| matches!(unit.faction, Faction::Player));
                return SimulationResult {
                    player_won,
                    units_alive: model.units.clone().into_iter().collect(),
                };
            }
        }
    }
}
