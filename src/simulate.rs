use geng::prelude::itertools::Itertools;
use std::time::Instant;
use std::{collections::BTreeMap, collections::VecDeque, path::PathBuf};

use crate::{
    assets::{self, Assets, ClanEffects, Config, GameRound, KeyMapping, Statuses},
    logic::{Events, Logic},
    model::MAX_TIER,
    model::{Faction, Model, Unit, UnitTemplate, UnitTemplates, UnitType},
    render::RenderModel,
    Clan,
};
use geng::prelude::*;

#[derive(clap::Args)]
pub struct Simulate {
    config_path: PathBuf,
}

#[derive(Deserialize, Clone, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
struct SimulationConfig {
    simulation: SimulationType,
    repeats: usize,
}

type RegexUnit = String;
type RegexClan = String;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
enum SimulationUnits {
    Units { units: Vec<RegexUnit> },
    Rounds { from: usize, to: usize },
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
enum SimulationType {
    Custom {
        player: Vec<RegexUnit>,
        opponent: SimulationUnits,
    },
    Balance {
        unit: RegexUnit,
    },
}

#[derive(Debug, Deserialize, Clone)]
struct BattleConfig {
    unit: Option<UnitType>,
    player: Vec<UnitType>,
    round: GameRound,
    repeats: usize,
    clans: HashMap<Clan, usize>,
    enemy_clans: HashMap<Clan, usize>,
    group: SimulationGroup,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum SimulationGroup {
    SameTier,
    UpperTier,
    LowerTier,
    Round,
    Enemies,
}

impl fmt::Display for SimulationGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimulationGroup::SameTier => write!(f, "SameTier"),
            SimulationGroup::UpperTier => write!(f, "UpperTier"),
            SimulationGroup::LowerTier => write!(f, "LowerTier"),
            SimulationGroup::Round => write!(f, "Round"),
            SimulationGroup::Enemies => write!(f, "Enemies"),
        }
    }
}

impl SimulationConfig {
    pub fn battles(
        self,
        rounds: &[GameRound],
        all_units: &Vec<UnitTemplate>,
        all_clans: &Vec<Clan>,
    ) -> Vec<BattleConfig> {
        match self.simulation.clone() {
            SimulationType::Balance { unit } => self.balance_battles(unit, all_units),
            SimulationType::Custom { player, opponent } => {
                self.custom_battles(player, opponent, rounds, all_units, all_clans)
            }
        }
    }

    fn balance_battles(&self, unit: RegexUnit, all_units: &Vec<UnitTemplate>) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        let units = self.to_templates(unit, all_units);
        for unit in units {
            battles.append(&mut self.same_tier(&unit, all_units));
            battles.append(&mut self.lower_tier(&unit, all_units));
            battles.append(&mut self.upper_tier(&unit, all_units));
        }
        battles
    }

    fn same_tier(&self, unit: &UnitTemplate, all_units: &Vec<UnitTemplate>) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        let same_tier = all_units.into_iter().filter(|other| {
            other.tier == unit.tier
                && (other.triple.is_some() && unit.triple.is_some()
                    || other.triple.is_none() && unit.triple.is_none())
        });
        for enemy in same_tier {
            let round = GameRound {
                name: "".to_string(),
                statuses: vec![],
                enemies: vec![enemy.name.clone()],
            };

            (1..=6).for_each(|i| {
                unit.clans.clone().into_iter().for_each(|clan| {
                    enemy.clans.clone().into_iter().for_each(|enemy_clan| {
                        battles.push(BattleConfig {
                            unit: Some(unit.name.clone()),
                            player: vec![unit.name.clone()],
                            round: round.clone(),
                            repeats: self.repeats,
                            clans: hashmap! {clan => i},
                            enemy_clans: hashmap! {enemy_clan => i},
                            group: SimulationGroup::SameTier,
                        })
                    })
                });
            });
        }
        battles
    }

    fn upper_tier(&self, unit: &UnitTemplate, all_units: &Vec<UnitTemplate>) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        if unit.tier == MAX_TIER {
            return battles;
        };

        let first_tier = all_units.into_iter().filter(|other| {
            other.tier == 1
                && (other.triple.is_some() && unit.triple.is_some()
                    || other.triple.is_none() && unit.triple.is_none())
        });
        let upper_tier = all_units.into_iter().filter(|other| {
            other.tier == unit.tier + 1
                && (other.triple.is_some() && unit.triple.is_some()
                    || other.triple.is_none() && unit.triple.is_none())
        });
        for enemy in upper_tier {
            let round = GameRound {
                name: "".to_string(),
                statuses: vec![],
                enemies: vec![enemy.name.clone()],
            };
            for ally in first_tier.clone() {
                (1..=6).for_each(|i| {
                    unit.clans.clone().into_iter().for_each(|clan| {
                        enemy.clans.clone().into_iter().for_each(|enemy_clan| {
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![unit.name.clone(), ally.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::UpperTier,
                            });
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![ally.name.clone(), unit.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::UpperTier,
                            });
                        })
                    });
                });
            }
        }
        battles
    }

    fn lower_tier(&self, unit: &UnitTemplate, all_units: &Vec<UnitTemplate>) -> Vec<BattleConfig> {
        let mut battles: Vec<BattleConfig> = vec![];
        if unit.tier == 1 {
            return battles;
        };

        let first_tier = all_units.into_iter().filter(|other| {
            other.tier == 1
                && (other.triple.is_some() && unit.triple.is_some()
                    || other.triple.is_none() && unit.triple.is_none())
        });
        let lower_tier = all_units.into_iter().filter(|other| {
            other.tier == unit.tier - 1
                && (other.triple.is_some() && unit.triple.is_some()
                    || other.triple.is_none() && unit.triple.is_none())
        });
        for enemy in lower_tier.clone() {
            for second_enemy in first_tier.clone() {
                (1..=6).for_each(|i| {
                    unit.clans.clone().into_iter().for_each(|clan| {
                        enemy.clans.clone().into_iter().for_each(|enemy_clan| {
                            let round = GameRound {
                                name: "".to_string(),
                                statuses: vec![],
                                enemies: vec![enemy.name.clone(), second_enemy.name.clone()],
                            };
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![unit.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::LowerTier,
                            });
                            let round = GameRound {
                                name: "".to_string(),
                                statuses: vec![],
                                enemies: vec![second_enemy.name.clone(), enemy.name.clone()],
                            };
                            battles.push(BattleConfig {
                                unit: Some(unit.name.clone()),
                                player: vec![unit.name.clone()],
                                round: round.clone(),
                                repeats: self.repeats,
                                clans: hashmap! {clan => i},
                                enemy_clans: hashmap! {enemy_clan => i},
                                group: SimulationGroup::LowerTier,
                            });
                        })
                    });
                });
            }
        }
        battles
    }

    fn custom_battles(
        self,
        player: Vec<RegexUnit>,
        opponent: SimulationUnits,
        rounds: &[GameRound],
        all_units: &Vec<UnitTemplate>,
        all_clans: &Vec<Clan>,
    ) -> Vec<BattleConfig> {
        let mut player_variants = vec![];
        player_variants = self.match_units(&all_units, &player, 0, player_variants);

        let battle_rounds: (SimulationGroup, Vec<GameRound>) = match &opponent {
            SimulationUnits::Units { units } => {
                let mut unit_vars = vec![];
                unit_vars = self.match_units(&all_units, &units, 0, unit_vars);
                let mut game_rounds = vec![];
                for variant in unit_vars {
                    game_rounds.push(GameRound {
                        name: "".to_string(),
                        statuses: vec![],
                        enemies: variant
                            .into_iter()
                            .map(|template| template.name.clone())
                            .collect(),
                    });
                }
                (SimulationGroup::Enemies, game_rounds)
            }
            SimulationUnits::Rounds { from, to } => (
                SimulationGroup::Round,
                rounds.iter().take(*to).skip(from - 1).cloned().collect(),
            ),
        };

        player_variants
            .into_iter()
            .flat_map(|player| {
                let mut rounds = vec![];
                for round in &battle_rounds.1 {
                    player.clone().into_iter().for_each(|unit| {
                        (1..=6).for_each(|i| {
                            unit.clans.clone().into_iter().for_each(|clan| {
                                rounds.push(BattleConfig {
                                    unit: None,
                                    player: player
                                        .clone()
                                        .into_iter()
                                        .map(|template| template.name)
                                        .collect(),
                                    round: round.clone(),
                                    repeats: self.repeats,
                                    clans: hashmap! {clan => i},
                                    enemy_clans: hashmap! {},
                                    group: battle_rounds.0.clone(),
                                })
                            });
                        });
                    });
                }

                rounds
            })
            .collect()
    }

    fn match_units(
        &self,
        all_units: &Vec<UnitTemplate>,
        units: &Vec<RegexUnit>,
        index: usize,
        result: Vec<Vec<UnitTemplate>>,
    ) -> Vec<Vec<UnitTemplate>> {
        let mut cloned = result.clone();
        if index == units.len() {
            return cloned;
        }

        if cloned.is_empty() {
            cloned.push(vec![]);
        }

        let regex_units = self.to_templates(units[index].clone(), all_units);
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

    fn to_units(&self, unit: RegexUnit, all_units: &Vec<UnitTemplate>) -> Vec<UnitType> {
        let regex = regex::Regex::new(&unit).expect("Failed to parse a regular expression");
        all_units
            .iter()
            .filter(move |unit| regex.is_match(&unit.long_name))
            .map(|unit| unit.name.clone())
            .collect()
    }

    fn to_templates(&self, unit: RegexUnit, all_units: &Vec<UnitTemplate>) -> Vec<UnitTemplate> {
        let regex = regex::Regex::new(&unit).expect("Failed to parse a regular expression");
        all_units
            .iter()
            .filter(move |unit| regex.is_match(&unit.long_name))
            .cloned()
            .collect()
    }

    fn to_clans(
        &self,
        clan: HashMap<RegexClan, usize>,
        all_clans: &Vec<Clan>,
    ) -> Vec<HashMap<Clan, usize>> {
        let mut result: Vec<HashMap<Clan, usize>> = vec![];
        for (clan_regex, size) in clan {
            let regex =
                regex::Regex::new(&clan_regex).expect("Failed to parse a regular expression");
            let clans: Vec<Clan> = all_clans
                .into_iter()
                .filter(move |clan| regex.is_match(&clan.to_string()))
                .map(|clan| *clan)
                .collect();
            for clan in clans {
                let mut map: HashMap<Clan, usize> = hashmap! {};
                map.insert(clan, size);
                result.push(map)
            }
        }
        result
    }
}

#[derive(Debug, Serialize, Clone)]
struct BattleResult {
    unit: Option<UnitType>,
    team: Vec<UnitType>,
    round: GameRound,
    clans: HashMap<Clan, usize>,
    enemy_clans: HashMap<Clan, usize>,
    group: SimulationGroup,
    win: bool,
    units_alive: Vec<UnitType>,
}

type Group = String;
type TeamView = String;

#[derive(Debug, Serialize, Clone)]
struct SimulationView {
    player: TeamView,
    koef: f64,
    groups: BTreeMap<Group, ClansGroupView>,
}

#[derive(Debug, Serialize, Clone)]
struct ClansGroupView {
    koef: f64,
    clans: BTreeMap<String, f64>,
}

struct AvgCounter {
    count: i64,
    sum: f64,
}

impl AvgCounter {
    pub fn new() -> Self {
        Self { count: 0, sum: 0.0 }
    }
    pub fn avg(&self) -> f64 {
        self.sum / (self.count as f64)
    }
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

        let all_units = assets.units.iter().map(|entry| entry.1).cloned().collect();
        let mut all_clans = vec![];
        for (clan, effect) in &assets.clans.map {
            all_clans.push(clan.clone());
        }
        let battles = simulation_config
            .clone()
            .battles(&assets.rounds, &all_units, &all_clans);
        let battle_results: Vec<BattleResult> = battles
            .into_iter()
            .flat_map(|battle| {
                info!("Starting the battle: {battle:?}");
                let results: Vec<BattleResult> = (1..=battle.repeats)
                    .map(|i| {
                        let result = Simulation::new(
                            Config {
                                player: battle.player.clone(),
                                clans: battle.clans.clone(),
                                enemy_clans: battle.enemy_clans.clone(),
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
                        BattleResult {
                            unit: battle.unit.clone(),
                            team: battle.player.clone(),
                            round: battle.round.clone(),
                            clans: battle.clans.clone(),
                            enemy_clans: battle.enemy_clans.clone(),
                            group: battle.group.clone(),
                            win: result.player_won,
                            units_alive: result
                                .units_alive
                                .into_iter()
                                .map(|unit| unit.unit_type)
                                .collect(),
                        }
                    })
                    .collect();
                results
            })
            .collect();

        info!("Simulation ended: {:?}", start.elapsed());
        let total_battles = battle_results.len();
        let result_path = PathBuf::new().join("simulation_result");
        let date_path = result_path.join(format!("{:?}", chrono::offset::Utc::now()));
        //let battles_path = date_path.join("battles");

        // Create directories
        match std::fs::create_dir_all(&date_path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }

        let mut result: Vec<SimulationView> = match simulation_config.simulation {
            SimulationType::Balance { unit } => result_balance(battle_results),
            SimulationType::Custom { player, opponent } => result_custom(battle_results),
        };

        result.sort_by(|a, b| b.koef.partial_cmp(&a.koef).unwrap());

        // Write results
        write_to(date_path.join("result.json"), &result).expect("Failed to write results");

        info!("Results saved: {:?}", start.elapsed());
    }
}

fn result_custom(battle_results: Vec<BattleResult>) -> Vec<SimulationView> {
    let mut balance: Vec<SimulationView> = vec![];
    let mut counters: HashMap<TeamView, HashMap<Group, HashMap<String, AvgCounter>>> = hashmap! {};
    let mut i = 0;
    battle_results.into_iter().for_each(|battle| {
        let group = if battle.group == SimulationGroup::Round {
            format!("{}: {:?}", battle.round.name, battle.round.enemies)
        } else {
            format!("{:?}", battle.round.enemies)
        };
        let units = counters
            .entry(format!("{:?}", battle.team))
            .or_insert(hashmap! {});
        let group = units.entry(group).or_insert(hashmap! {});
        let clans = group
            .entry(format!("{:?}", battle.clans))
            .or_insert(AvgCounter::new());
        if battle.win {
            clans.sum += 1.0;
        };
        clans.count += 1;
    });

    for (team, counters) in counters {
        let groups: BTreeMap<Group, ClansGroupView> = counters
            .iter()
            .map(|(key, value)| {
                let clans: BTreeMap<String, f64> = value
                    .iter()
                    .map(|(key, value)| (key.clone(), value.avg()))
                    .collect();
                (
                    key.to_string(),
                    ClansGroupView {
                        koef: clans.values().sum::<f64>() / value.values().len() as f64,
                        clans,
                    },
                )
            })
            .collect();
        let koef =
            groups.values().map(|value| value.koef).sum::<f64>() / groups.values().len() as f64;
        balance.push(SimulationView {
            player: team,
            koef,
            groups,
        });
    }
    balance
}

fn result_balance(battle_results: Vec<BattleResult>) -> Vec<SimulationView> {
    let mut balance: Vec<SimulationView> = vec![];
    let mut counters: HashMap<TeamView, HashMap<Group, HashMap<String, AvgCounter>>> = hashmap! {};

    battle_results.into_iter().for_each(|battle| {
        let units = counters.entry(battle.unit.unwrap()).or_insert(hashmap! {});
        let group = units.entry(battle.group.to_string()).or_insert(hashmap! {});
        let clans = group
            .entry(format!("{:?} VS {:?}", battle.clans, battle.enemy_clans))
            .or_insert(AvgCounter::new());
        if battle.win {
            clans.sum += 1.0;
        };
        clans.count += 1;
    });

    for (unit, counters) in counters {
        let groups: BTreeMap<Group, ClansGroupView> = counters
            .iter()
            .map(|(key, value)| {
                let clans: BTreeMap<String, f64> = value
                    .iter()
                    .map(|(key, value)| (key.clone(), value.avg()))
                    .collect();
                (
                    key.clone(),
                    ClansGroupView {
                        koef: clans.values().sum::<f64>() / value.values().len() as f64,
                        clans,
                    },
                )
            })
            .collect();
        let koef =
            groups.values().map(|value| value.koef).sum::<f64>() / groups.values().len() as f64;
        balance.push(SimulationView {
            player: unit,
            koef,
            groups,
        });
    }
    balance
}

fn write_to<T: Serialize>(path: impl AsRef<std::path::Path>, item: &T) -> std::io::Result<()> {
    let path = path.as_ref();
    let file = std::fs::File::create(path).expect(&format!("Failed to create {path:?}"));
    let data = serde_json::to_string_pretty(item).expect("Failed to serialize item");
    std::fs::write(path, data)?;
    Ok(())
}

struct SimulationResult {
    player_won: bool,
    units_alive: Vec<Unit>,
}

struct Simulation {
    config: Config,
    key_mappings: Vec<KeyMapping>,
    model: Model,
    delta_time: f64,
    // TODO: time or steps limit
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
                1.0,
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
