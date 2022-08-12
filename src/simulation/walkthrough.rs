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
    unit_picks: Vec<usize>,
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

        let all_units: Vec<UnitTemplate> =
            assets.units.iter().map(|entry| entry.1).cloned().collect();
        let mut walkthrough_results: HashMap<String, String> = hashmap! {};

        let mut hero_picks:HashMap<UnitType, usize> = hashmap! {};
        let mut hero_picks_last:HashMap<UnitType, usize> = hashmap! {};
        let mut end_rounds:HashMap<String, usize> = hashmap! {};

        for index in 0..walkthrough_config.repeats {
            let mut tier = 1;
            let mut round_index = 0;
            let mut player: Vec<UnitType> = vec![];
            let mut shop_updates = walkthrough_config.shop_updates.clone();
            let mut next_update_round = shop_updates.pop_front();
            let mut results: Vec<BattleResult> = vec![];
            let mut inventory_units: VecDeque<UnitTemplate> = VecDeque::new();
            let mut lives = walkthrough_config.lives;
            for round in &assets.rounds {
                let count = if walkthrough_config.unit_picks.len() <= round_index {
                    walkthrough_config.unit_picks[walkthrough_config.unit_picks.len() - 1]
                } else {
                    walkthrough_config.unit_picks[round_index]
                };

                let variants = Self::shop_variants(
                    &player,
                    count,
                    tier,
                    all_units.clone(),
                    &mut inventory_units,
                );
                let mut best_win: Option<BattleResult> = None;
                let mut best_lose: Option<BattleResult> = None;

                variants.into_iter().for_each(|variant| {
                    let clans = Self::calc_clan_members(&variant);
                    let result = Battle::new(
                        Config {
                            player: variant
                                .into_iter()
                                .map(|unit| unit.name)
                                .collect::<Vec<UnitType>>(),
                            clans,
                            ..config.clone()
                        },
                        assets.clans.clone(),
                        assets.statuses.clone(),
                        round.clone(),
                        assets.units.clone(),
                        0.02 as f64,
                        lives,
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
                                if lose.units_alive.len() < result.units_alive.len() {
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
                    lives += battle_result.damage_sum;
                }
                
                if lives <= 0 {
                    results.push(battle_result);
                    break;
                }

                round_index += 1;
                if let Some(next_update_round_unwrap) = next_update_round {
                    if next_update_round_unwrap == round_index + 1 {
                        tier += 1;
                        next_update_round = shop_updates.pop_front();
                    }
                }
                player = battle_result.player.clone();
                let alives: Vec<UnitType> = battle_result.units_alive.clone();
                let inventory: Vec<UnitType> = inventory_units
                    .clone()
                    .into_iter()
                    .map(|unit| unit.name.clone())
                    .collect();
                info!(
                "\nLives: {} \nPlayer team:{:?} \nUnits alive:{:?} \nInventory:{:?} \nRound: {} \nTier: {}",
                lives, player, alives, inventory, round_index, tier - 1 
            );
                results.push(battle_result);
            }
            let mut lost_lives = "".to_owned();
            let mut last_result = None;
            let mut lives = walkthrough_config.lives;

            results.clone().into_iter().for_each(|result| {
                result.player.clone()
                .into_iter()
                .for_each(|unit| *hero_picks.entry(unit).or_insert(0)+=1);
                
                last_result = Some(result.clone());
                lost_lives.push_str(
                    format!(
                        "({}:{}:{}) ",
                        result.round,
                        result.damage_sum,
                        result.health_sum
                    )
                    .as_str(),
                );
                if result.damage_sum < 0{
                    lives += result.damage_sum;
                }
            });
            let last_result = last_result.unwrap().clone();
            last_result.player.clone()
                .into_iter()
                .for_each(|unit| *hero_picks_last.entry(unit).or_insert(0)+=1);
            *end_rounds.entry(result.round.clone()).or_insert(0)+=1;
            walkthrough_results.insert(
                format!("{:?}", last_result.player),
                format!(
                    "{}, Lives: {} {}",
                    last_result.round, lives, lost_lives
                ),
            );
        }

        info!("Walkthrough ended: {:?}", start.elapsed());
        let result_path = PathBuf::new().join("simulation_result");
        let date_path = result_path.join(format!("{:?}", chrono::offset::Utc::now()));

        // Create directories
        match std::fs::create_dir_all(&result_path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }
        match std::fs::create_dir_all(&date_path) {
            Ok(()) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => panic!("Failed to create a simulation_result directory: {error}"),
            },
        }

        // Write results
        write_to(date_path.join("result.json"), &walkthrough_results)
            .expect("Failed to write results");
        write_to(date_path.join("hero_picks.json"), &hero_picks)
            .expect("Failed to write results");
            write_to(date_path.join("hero_picks_last.json"), &hero_picks_last)
            .expect("Failed to write results");
            write_to(date_path.join("end_rounds.json"), &end_rounds)
            .expect("Failed to write results");

        info!("Results saved: {:?}", start.elapsed());
    }

    fn shop_variants(
        player: &Vec<UnitType>,
        count: usize,
        tier: usize,
        all_units: Vec<UnitTemplate>,
        inventory_units: &mut VecDeque<UnitTemplate>,
    ) -> Vec<Vec<UnitTemplate>> {
        let mut result: Vec<Vec<UnitTemplate>> = vec![];
        let player: Vec<UnitTemplate> = player
            .into_iter()
            .map(|unit| {
                all_units
                    .clone()
                    .into_iter()
                    .find(|template| *template.name == *unit)
                    .unwrap()
            })
            .collect();

        if count == 0 {
            result.push(player.clone());
            return result;
        }

        let units_count = TIER_UNITS[tier - 1];
        let mut shop_units = all_units
            .clone()
            .into_iter()
            .filter(|unit| unit.tier <= tier as u32 && unit.triple.is_some())
            .choose_multiple(&mut global_rng(), units_count);

        if shop_units.len() <= count {
            shop_units.append(&mut player.clone());
            shop_units.sort_by(|a, b| Ord::cmp(&b.range, &a.range));
            result.push(shop_units);
        } else {
            shop_units
                .into_iter()
                .combinations(count)
                .for_each(|mut variant| {
                    variant.append(&mut player.clone());
                    variant.append(&mut inventory_units.iter().cloned().collect());
                    inventory_units.clear();
                    variant = Self::check_triple(variant, all_units.clone());
                    variant = Self::check_max_slots(variant, inventory_units);
                    variant.sort_by(|a, b| Ord::cmp(&b.range, &a.range));
                    result.push(variant);
                })
        }

        result
    }

    fn check_max_slots(
        team: Vec<UnitTemplate>,
        inventory_units: &mut VecDeque<UnitTemplate>,
    ) -> Vec<UnitTemplate> {
        let delete_count = team.len() as i32 - (SIDE_SLOTS as i32);
        if delete_count > 0 {
            let mut sorted = team.clone();
            sorted.sort_by(|a, b| {
                b.tier
                    .cmp(&(a.tier + if a.triple.is_none() { 1 } else { 0 }))
            });
            (0..delete_count).into_iter().for_each(|i| {
                if inventory_units.len() == MAX_INVENTORY {
                    inventory_units.pop_back();
                }
                inventory_units.push_front(sorted.pop().unwrap());
            });
            sorted
        } else {
            team
        }
    }

    fn check_triple(team: Vec<UnitTemplate>, all_units: Vec<UnitTemplate>) -> Vec<UnitTemplate> {
        let mut result: Vec<UnitTemplate> = vec![];
        let mut counts: HashMap<UnitType, (usize, UnitTemplate)> = hashmap! {};
        team.into_iter()
            .for_each(|unit| counts.entry(unit.name.clone()).or_insert((0, unit)).0 += 1);
        counts.into_values().for_each(|(count, unit)| {
            let mut count = count;
            while unit.triple.is_some() && count >= 3 {
                count -= 3;
                let triple = all_units
                    .clone()
                    .into_iter()
                    .find(|template| *template.name == unit.triple.as_ref().unwrap().clone())
                    .unwrap();
                result.push(triple);
            }
            (0..count).for_each(|i| result.push(unit.clone()));
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
