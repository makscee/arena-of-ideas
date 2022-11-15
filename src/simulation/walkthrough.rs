use crate::Position;
use std::{fs::create_dir, vec};

use geng::prelude::itertools::sorted;

use super::*;
use crate::{model::SIDE_SLOTS, simulation::battle::BattleResult};

const TIER_UNITS: [usize; 6] = [3, 4, 4, 5, 5, 6];
const MAX_INVENTORY: usize = 10;

#[derive(clap::Args)]
pub struct Walkthrough {
    config_path: PathBuf,
}

#[derive(Deserialize, Clone, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
struct WalkthroughConfig {
    shop_updates: VecDeque<usize>,
    unit_picks: Vec<(usize, usize)>,
    lives: i32,
    repeats: usize,
}
impl Walkthrough {
    pub fn run(self, geng: &Geng, assets: Assets, mut config: Config) {
        let start = Instant::now();
        let config_path = static_path().join(self.config_path);
        let walkthrough_config = futures::executor::block_on(
            <WalkthroughConfig as geng::LoadAsset>::load(geng, &config_path),
        )
        .unwrap();
        info!("Starting walkthrough");

        let all_units: Vec<UnitTemplate> = assets
            .units
            .iter()
            .filter(|unit| unit.1.tier > 0)
            .map(|entry| entry.1)
            .cloned()
            .collect();
        let mut walkthrough_results: HashMap<String, String> = hashmap! {};

        let mut hero_rounds: HashMap<UnitType, usize> = hashmap! {};
        let mut hero_picks_last: HashMap<UnitType, usize> = hashmap! {};
        let mut hero_picks: HashMap<UnitType, usize> = hashmap! {};
        let mut end_rounds: HashMap<String, usize> = hashmap! {};
        let mut round_damages: HashMap<String, (i32, i32)> = hashmap! {};

        // Create directories
        let result_path = PathBuf::new().join("simulation_result");
        let date_path = result_path.join(format!("{:?}", chrono::offset::Utc::now()));

        Self::create_dir(&result_path);
        Self::create_dir(&date_path);

        for index in 0..walkthrough_config.repeats {
            let current_simulation_path = date_path.join(format!("Simulation#{}", index + 1));
            let battles_path = current_simulation_path.join("battles");
            Self::create_dir(&current_simulation_path);
            Self::create_dir(&battles_path);
            let walkthrough_start = Instant::now();
            let mut tier = 1;
            let mut round_index = 0;
            let mut player: Vec<Unit> = vec![];
            let mut shop_updates = walkthrough_config.shop_updates.clone();
            let mut next_update_round = shop_updates.pop_front();
            let mut results: Vec<BattleResult> = vec![];
            let mut lives = walkthrough_config.lives;
            for round in &assets.rounds {
                let round_start = Instant::now();
                let count = if walkthrough_config.unit_picks.len() <= round_index {
                    walkthrough_config.unit_picks[walkthrough_config.unit_picks.len() - 1]
                } else {
                    walkthrough_config.unit_picks[round_index]
                };

                let variants =
                    Self::shop_variants(&player, count.0, count.1, tier, all_units.clone());
                let mut best_win: Option<BattleResult> = None;
                let mut best_lose: Option<BattleResult> = None;
                variants.into_iter().for_each(|variant| {
                    let clans = Self::calc_clan_members(
                        &variant.iter().map(|unit| unit.template.clone()).collect(),
                    );
                    let result = Battle::new(
                        Config {
                            player: vec![],
                            clans,
                            ..config.clone()
                        },
                        assets.clans.clone(),
                        assets.statuses.clone(),
                        round.clone(),
                        assets.units.clone(),
                        0.02,
                        lives,
                        variant,
                    )
                    .run();
                    if result.player_won {
                        best_win = match &best_win {
                            None => Some(result),
                            Some(win) => {
                                if win.units_alive.len() < result.units_alive.len() {
                                    Some(result)
                                } else {
                                    Some(win.clone())
                                }
                            }
                        };
                    } else {
                        best_lose = match &best_lose {
                            None => Some(result),
                            Some(lose) => {
                                if lose.units_alive.len() > result.units_alive.len() {
                                    Some(result)
                                } else {
                                    Some(lose.clone())
                                }
                            }
                        };
                    }
                });
                let battle_result = best_win.unwrap_or_else(|| best_lose.unwrap());
                if battle_result.damage_sum < 0 {
                    lives -= 1;
                }

                if lives <= 0 {
                    results.push(battle_result);
                    break;
                }

                round_index += 1;
                let current_tier = tier;
                if let Some(next_update_round_unwrap) = next_update_round {
                    if next_update_round_unwrap == round_index + 1 {
                        tier += 1;
                        next_update_round = shop_updates.pop_front();
                    }
                }

                let s1: HashSet<UnitType> = battle_result
                    .player
                    .iter()
                    .map(|unit| unit.unit_type.clone())
                    .collect();
                let s2: HashSet<UnitType> =
                    player.iter().map(|unit| unit.unit_type.clone()).collect();
                (&s1 - &s2).iter().cloned().for_each(|unit| {
                    *hero_picks.entry(unit).or_insert(0) += 1;
                });
                player = battle_result.player.clone();
                let enemy = battle_result.enemy.clone();
                let alives: Vec<UnitType> = battle_result.units_alive.clone();
                let player_types: Vec<UnitType> = battle_result
                    .player
                    .iter()
                    .map(|unit| unit.unit_type.clone())
                    .collect();
                let log = format!(
                    "Walkthrough:{}/{}, Round: {}, Time: {:?}, Lives: {}, Tier: {}\nStats Before:{:?} \nPlayer: {:?} \nEnemy: {:?}\nAlives: {:?}\nStats after:{:?} ",
                    index + 1, walkthrough_config.repeats, round_index, round_start.elapsed(), lives,  current_tier, battle_result.stats_before, player_types, enemy,  alives, battle_result.stats_after
                    );

                write_to(battles_path.join(format!("{}.txt", round.name)), &log)
                    .expect("Failed to write results");
                results.push(battle_result);
            }
            let mut lost_lives = "".to_owned();
            let mut last_result = None;
            let mut lives = walkthrough_config.lives;

            results.clone().into_iter().for_each(|result| {
                result
                    .player
                    .clone()
                    .into_iter()
                    .for_each(|unit| *hero_rounds.entry(unit.unit_type).or_insert(0) += 1);

                last_result = Some(result.clone());
                lost_lives.push_str(
                    format!(
                        "({} D:{} H:{})",
                        result.round, result.damage_sum, result.health_sum
                    )
                    .as_str(),
                );
                let round_damage = round_damages.entry(result.round).or_insert((0, 0));
                round_damage.0 += result.damage_sum;
                round_damage.1 += result.health_sum;
                if result.damage_sum < 0 {
                    lives -= 1;
                }
            });
            let last_result = last_result.unwrap().clone();
            last_result
                .player
                .clone()
                .into_iter()
                .for_each(|unit| *hero_picks_last.entry(unit.unit_type).or_insert(0) += 1);
            *end_rounds.entry(last_result.round.clone()).or_insert(0) += 1;
            let last_result_types: Vec<UnitType> = last_result
                .player
                .iter()
                .map(|unit| unit.unit_type.clone())
                .collect();
            walkthrough_results.insert(
                format!("{:?}", last_result_types),
                format!("{}, Lives: {} {}", last_result.round, lives, lost_lives),
            );
            let time = walkthrough_start.elapsed();
            let all_time = start.elapsed();
            let time_remaining = (all_time / ((index as u32) + 1))
                * (walkthrough_config.repeats - (index + 1)) as u32;
            warn!(
                "Walkthrough:{}/{} \nTime: {:?}\nTime remaining: {:?}",
                index + 1,
                walkthrough_config.repeats,
                time,
                time_remaining
            );
        }
        warn!("Walkthrough ended: {:?}", start.elapsed());

        let mut round_damage_string = "".to_owned();
        round_damages
            .clone()
            .into_iter()
            .sorted_by(|a, b| b.1 .1.cmp(&a.1 .1))
            .for_each(|(k, v)| {
                round_damage_string.push_str(
                    format!(
                        "{}: {}:{}\n",
                        k,
                        v.0 / (walkthrough_config.repeats as i32),
                        v.1 / (walkthrough_config.repeats as i32)
                    )
                    .as_str(),
                );
            });

        // Add not picked heroes
        all_units.iter().for_each(|unit| {
            if !hero_rounds.contains_key(&unit.name) {
                hero_rounds.insert(unit.name.clone(), 0);
            }
            if !hero_picks.contains_key(&unit.name) {
                hero_picks.insert(unit.name.clone(), 0);
            }
            if !hero_picks_last.contains_key(&unit.name) {
                hero_picks_last.insert(unit.name.clone(), 0);
            }
        });

        // Write results
        write_to(date_path.join("result.json"), &walkthrough_results)
            .expect("Failed to write results");
        write_to_file(
            date_path.join("hero_rounds.txt"),
            &Self::to_file(&hero_rounds),
        )
        .expect("Failed to write results");
        write_to_file(
            date_path.join("hero_picks.txt"),
            &Self::to_file(&hero_picks),
        )
        .expect("Failed to write results");
        write_to_file(
            date_path.join("hero_picks_last.txt"),
            &Self::to_file(&hero_picks_last),
        )
        .expect("Failed to write results");
        write_to_file(
            date_path.join("end_rounds.txt"),
            &Self::to_file(&end_rounds),
        )
        .expect("Failed to write results");
        write_to_file(date_path.join("round_damages.txt"), &round_damage_string)
            .expect("Failed to write results");

        info!("Results saved: {:?}", start.elapsed());
    }

    fn create_dir(path: &PathBuf) {
        match std::fs::create_dir_all(path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }
    }

    fn to_file(map: &HashMap<String, usize>) -> String {
        let mut result = "".to_owned();
        map.clone()
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .for_each(|(k, v)| {
                result.push_str(format!("{}: {}\n", k, v).as_str());
            });
        result
    }

    fn shop_variants(
        player: &Vec<Unit>,
        count: usize,
        rerolls: usize,
        tier: usize,
        all_units: Vec<UnitTemplate>,
    ) -> Vec<Vec<Unit>> {
        let mut result: Vec<Vec<Unit>> = vec![];

        if count == 0 {
            result.push(player.clone());
            return result;
        }

        let units_count = TIER_UNITS[tier - 1] + (TIER_UNITS[tier - 1] * rerolls);
        let statuses = Statuses { map: hashmap! {} };
        let mut shop_units: Vec<Unit> = all_units
            .clone()
            .into_iter()
            .filter(|unit| unit.tier <= tier)
            .choose_multiple(&mut global_rng(), units_count)
            .iter()
            .map(|template| Unit::new(template, 0, Position::zero(Faction::Player), &statuses))
            .collect();
        if shop_units.len() <= count {
            shop_units.append(&mut player.clone());
            result.push(shop_units);
        } else {
            shop_units
                .into_iter()
                .combinations(count)
                .for_each(|mut variant| {
                    variant.append(&mut player.clone());
                    variant = Self::check_stackable(variant, all_units.clone());
                    let variants = Self::check_max_slots(variant);
                    variants.into_iter().for_each(|variant| {
                        result.push(variant);
                    });
                })
        }
        result
    }

    fn check_max_slots(team: Vec<Unit>) -> Vec<Vec<Unit>> {
        if team.len() <= SIDE_SLOTS {
            return vec![team.clone()];
        }
        team.into_iter().combinations(SIDE_SLOTS).collect()
    }

    fn check_stackable(team: Vec<Unit>, all_units: Vec<UnitTemplate>) -> Vec<Unit> {
        let mut result: Vec<Unit> = vec![];
        let mut counts: HashMap<UnitType, Vec<Unit>> = hashmap! {};
        team.into_iter().for_each(|unit| {
            counts
                .entry(unit.unit_type.clone())
                .or_insert(vec![])
                .push(unit.clone())
        });
        counts.into_values().for_each(|stacks| {
            let mut unit = stacks.first().unwrap().clone();
            let mut count = stacks.len();
            let mut unit_to_add = 0;
            stacks.iter().for_each(|unit_to_merge| {
                if unit.id != unit_to_merge.id {
                    if !unit.merge(unit_to_merge.clone()) {
                        result.push(unit.clone());
                        unit = unit_to_merge.clone();
                    }
                }
            });
            result.push(unit.clone());
        });
        result
    }

    fn calc_clan_members(units: &Vec<UnitTemplate>) -> HashMap<Clan, usize> {
        let unique_units = units
            .into_iter()
            .map(|unit| (&unit.name, &unit.clans))
            .collect::<HashMap<_, _>>();
        let mut clans = HashMap::new();
        for clan in unique_units.into_values().flatten() {
            *clans.entry(clan.clone()).or_insert(0) += 1;
        }
        clans
    }
}
