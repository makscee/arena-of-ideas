use geng::prelude::file::load_json;

use super::*;

pub struct RatingSystem {}

impl RatingSystem {
    pub fn simulate_walkthrough(world: &mut legion::World, resources: &mut Resources) {
        resources.logger.set_enabled(false);
        let mut i: usize = 0;
        let mut ratings: Ratings = default();
        let mut levels: Vec<usize> = vec![0; resources.ladder.count() + 1];
        loop {
            i += 1;
            let run_timer = Instant::now();
            let (level_reached, total_score, team, pick_data, score_data) =
                Self::simulate_run(world, resources);
            *levels.get_mut(level_reached).unwrap() += 1;
            for (name, data) in score_data {
                ratings.add_rating(&name, RatingType::Score, data.0, data.1);
            }
            for (name, data) in pick_data {
                ratings.add_rating(&name, RatingType::PickRate, data.0, data.1);
            }

            ratings.calculate();
            println!("{ratings}");

            for (i, name) in resources
                .ladder
                .teams
                .iter()
                .map(|x| x.name.clone())
                .chain(Some("Game Over".to_string()))
                .enumerate()
            {
                println!("{} {} = {}", i, name, levels.get(i).unwrap());
            }
            println!(
                "Run #{} took {:?} reached {} {}",
                i,
                run_timer.elapsed(),
                level_reached,
                team
            );
        }
    }

    fn simulate_run(
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> (
        usize,
        usize,
        PackedTeam,
        HashMap<String, (usize, usize)>,
        HashMap<String, (usize, usize)>,
    ) {
        let pool: HashMap<String, PackedUnit> = HashMap::from_iter(
            resources
                .hero_pool
                .all()
                .into_iter()
                .map(|x| (x.name.clone(), x)),
        );

        let mut team = PackedTeam::new("light".to_string(), vec![]);
        ShopSystem::init_game(world, resources);
        let buy_price = ShopSystem::buy_price(world);
        let sell_price = ShopSystem::sell_price(world);
        const MAX_ARRANGE_TRIES: usize = 5;
        let mut pick_show_count = HashMap::default();
        let mut score_games_count = HashMap::default();
        let mut total_result = 0;
        loop {
            let extra_units = {
                let mut value = ShopSystem::floor_money(resources.ladder.current_ind()) / buy_price;
                value += value * sell_price / buy_price;
                value
            } as usize;
            let dark = Ladder::load_team(resources);
            let max_slots = (resources.ladder.current_ind() + resources.options.initial_team_slots)
                .min(MAX_SLOTS);
            team.slots = max_slots;

            let shop_case = pool
                .values()
                .choose_multiple(&mut thread_rng(), MAX_SLOTS * extra_units);
            for shop_unit in shop_case.iter() {
                let (pick, show) = pick_show_count.remove(&shop_unit.name).unwrap_or_default();
                pick_show_count.insert(shop_unit.name.clone(), (pick, show + 1));
            }
            let mut battle_result = 0;
            let mut candidate = None;
            let mut picked = Vec::default();
            for _ in 0..MAX_ARRANGE_TRIES {
                let mut new_units = vec![];
                for _ in 0..extra_units {
                    new_units.push(*shop_case.choose(&mut thread_rng()).unwrap());
                }
                let team_entity = team.unpack(&Faction::Team, world, resources);
                Event::ShopEnd.send(world, resources);
                Event::ShopStart.send(world, resources);
                let slots = (1..=max_slots).choose_multiple(&mut thread_rng(), extra_units);
                for (i, unit) in new_units.iter().enumerate() {
                    let slot = *slots.get(i).unwrap();
                    let entity = unit.unpack(world, resources, slot, None, Some(team_entity));
                    if team.units.len() + i < max_slots {
                        SlotSystem::make_gap(Faction::Team, slot, world, resources, None);
                    } else {
                        if let Some(entity) =
                            SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
                        {
                            ShopSystem::do_sell(entity, resources, world);
                        }
                    }
                    ShopSystem::do_buy(entity, slot, resources, world);
                    ActionSystem::run_ticks(world, resources, &mut None);
                }
                let new_team = PackedTeam::pack(&Faction::Team, world, resources);
                UnitSystem::clear_faction(world, resources, Faction::Team);
                let result = SimulationSystem::run_battle(&new_team, &dark, world, resources, None);
                resources.action_queue.clear();
                if result > battle_result {
                    candidate = Some(new_team);
                    picked = new_units.iter().map(|x| x.name.clone()).collect_vec();
                    battle_result = result;
                }
                if result == 3 {
                    break;
                }
            }
            if battle_result == 0 || !resources.ladder.next() {
                break;
            } else {
                team = candidate.unwrap();
                picked
                    .iter()
                    .for_each(|name| pick_show_count.get_mut(name).unwrap().0 += 1);
                for unit in team.units.iter() {
                    let mut data: (usize, usize) =
                        score_games_count.remove(&unit.name).unwrap_or_default();
                    data.0 += battle_result;
                    data.1 += 1;
                    score_games_count.insert(unit.name.clone(), data);
                }
                total_result += battle_result;
            }
        }
        let level_reached = resources.ladder.current_ind();
        resources.ladder.reset();
        (
            level_reached,
            total_result,
            team,
            pick_show_count,
            score_games_count,
        )
    }

    pub fn simulate_enemy_ratings_calculation(
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        resources.logger.set_enabled(false);

        let mut teams = EnemyPool::generate_teams();
        let mut ratings = Ratings::default();
        let mut cnt = 0;
        let save_path = static_path().join("levels.json");
        loop {
            Self::rate_teams(&teams, &mut ratings, world, resources);
            ratings.calculate();
            cnt += 1;
            teams.sort_by(|a, b| {
                ratings
                    .get_rating(&a.name)
                    .total_cmp(&ratings.get_rating(&b.name))
            });
            let save = serde_json::to_string_pretty(&teams).unwrap();

            match std::fs::write(&save_path, save) {
                Ok(_) => debug!("Levels saved to {:?}", &save_path),
                Err(error) => error!("Can't save: {}", error),
            }
            println!("\nRun#{cnt}{ratings}");
        }
    }

    pub fn rate_teams(
        teams: &Vec<PackedTeam>,
        ratings: &mut Ratings,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        for light in teams.iter() {
            for dark in teams.iter() {
                let mut dark = dark.clone();
                dark = Ladder::generate_team(dark);
                let result = SimulationSystem::run_battle(light, &dark, world, resources, None);
                ratings.add_rating(&light.name, RatingType::WinRate, result, 1);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Ratings {
    pub data: HashMap<String, (f64, HashMap<RatingType, (usize, usize)>)>,
}

#[derive(
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    enum_iterator::Sequence,
    Clone,
    Copy,
    Debug,
    Ord,
    PartialOrd,
)]
pub enum RatingType {
    PickRate,
    Score,
    WinRate,
}

impl Ratings {
    pub fn add_rating(&mut self, name: &str, rating: RatingType, score: usize, max: usize) {
        let mut data = self.data.remove(name).unwrap_or_default();
        let mut rating_data = data.1.remove(&rating).unwrap_or_default();
        rating_data.0 += score;
        rating_data.1 += max;
        data.1.insert(rating, rating_data);
        self.data.insert(name.to_string(), data);
    }

    pub fn get_rating(&self, name: &str) -> f64 {
        self.data.get(name).unwrap().0
    }

    pub fn calculate(&mut self) {
        let mut sorted: HashMap<RatingType, Vec<(String, f64)>> = default();
        for (name, (_, ratings)) in self.data.iter() {
            for (rating_type, (score, max)) in ratings.iter() {
                let mut v = sorted.remove(rating_type).unwrap_or_default();
                v.push((name.clone(), *score as f64 / *max as f64));
                sorted.insert(*rating_type, v);
            }
        }
        let mut results: HashMap<String, f64> = default();
        // assert!(sorted.iter().map(|x| x.1.len()).all_equal());
        for (_rating_type, v) in sorted.iter_mut() {
            for (ind, (name, _value)) in v.iter().sorted_by(|a, b| a.1.total_cmp(&b.1)).enumerate()
            {
                let data = results.remove(name).unwrap_or_default() + ind as f64;
                results.insert(name.clone(), data);
            }
        }
        for (name, result) in results {
            self.data.get_mut(&name).unwrap().0 = result;
        }
    }
}

impl Display for Ratings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spaces = "                                                                 ";
        let mut result: String = default();
        for (name, (score, ratings)) in self.data.iter().sorted_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
        {
            let mut name = name.clone();
            name.push_str(spaces);
            let (name, _) = name.split_at(50);
            result += &format!(
                "\n{name} {score} [{}]",
                ratings
                    .iter()
                    .sorted_by_key(|x| x.0)
                    .map(|(rating, (a, b))| format!(
                        "{rating:?}({a}/{b}) = {:.2}%",
                        *a as f32 / *b as f32 * 100.0
                    ))
                    .join(" ")
            );
        }
        write!(f, "{result}")
    }
}
