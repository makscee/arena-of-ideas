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
            HeroPool::all(&resources)
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
                let mut value = 10;
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
                        SlotSystem::make_gap(Faction::Team, slot, world, None);
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
                let new_team = PackedTeam::pack(Faction::Team, world, resources);
                UnitSystem::clear_faction(world, resources, Faction::Team);
                let (_, result) =
                    SimulationSystem::run_battle(&new_team, &dark, world, resources, None);
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

    pub fn simulate_hero_ratings_calculation(world: &mut legion::World, resources: &mut Resources) {
        resources.logger.set_enabled(false);

        let heroes = HeroPool::all(&resources);
        let mut teams: Vec<PackedTeam> = default();
        let mut team_ratings = Ratings::default();

        // fill teams with random max size teams
        // each team X random matches against other
        // retain top 1/3
        // sum end place of each hero
        // fill 2/3 with mutations
        // repeat
        const TEAMS: usize = 30;
        const SIZE: usize = 5;
        const MATCHES: usize = 20;
        for _ in 0..TEAMS * 3 {
            let mut units: Vec<PackedUnit> = default();
            for _ in 0..SIZE {
                let unit = heroes.choose(&mut thread_rng()).unwrap().clone();
                units.push(unit);
            }
            let team = PackedTeam::from_units(units);
            teams.push(team);
        }
        let mut heroes_rating: HashMap<String, (usize, usize)> = default();
        let mut top_teams: HashSet<String> = default();
        loop {
            team_ratings = default();
            Self::rate_teams(&teams, &mut team_ratings, MATCHES, world, resources);
            println!("{team_ratings}");
            teams.sort_by(|a, b| {
                team_ratings
                    .get_rating(&a.name)
                    .total_cmp(&team_ratings.get_rating(&b.name))
            });
            top_teams.insert(teams[0].name.to_owned());
            let old_teams = teams;
            teams = default();
            for (_, (score, _)) in heroes_rating.iter_mut() {
                *score = 0;
            }
            for (i, team) in old_teams.into_iter().rev().enumerate() {
                if i >= TEAMS {
                    debug!("-{i} {}", team.name);
                    continue;
                }
                for (i, unit) in team.units.iter().enumerate() {
                    let (score, count) = heroes_rating.entry(unit.name.to_owned()).or_default();
                    *score += TEAMS - i;
                    *count += 1;
                }
                let mut t1 = team.clone();
                Self::mutate(&mut t1, &heroes);
                let mut t2 = team.clone();
                Self::mutate(&mut t2, &heroes);
                Self::mutate(&mut t2, &heroes);
                teams.push(team);
                teams.push(t1);
                teams.push(t2);
            }
            let heroes_rating = heroes_rating
                .iter()
                .sorted_by_key(|(_, (score, count))| (*score, *count))
                .map(|(name, (score, count))| (name, score, count))
                .collect_vec();
            let ratings_json = heroes_rating
                .iter()
                .enumerate()
                .map(|(ind, (name, _, _))| format!("\"{name}\":{ind}"))
                .join(",");
            println!("Ratings json:\n{{{ratings_json}}}");
            println!("\nTop teams:\n{}", top_teams.iter().join("\n"));
            let heroes_rating = heroes_rating
                .iter()
                .map(|(name, score, count)| format!("{score}/{count}  {name}"))
                .join("\n");
            println!("\nHeroes rating:\n{heroes_rating}");
        }
    }

    fn mutate(team: &mut PackedTeam, heroes: &Vec<PackedUnit>) {
        let before = team.name.clone();
        let rng = &mut thread_rng();
        team.units.remove(rng.gen_range(0..team.units.len()));
        team.units.insert(
            rng.gen_range(0..=team.units.len()),
            heroes.choose(rng).unwrap().clone(),
        );
        team.generate_name();
        debug!("mutate: {before} -> {}", team.name);
    }

    pub fn simulate_enemy_ratings_calculation(
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        resources.logger.set_enabled(false);

        println!("Enter teams count:");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line = line.trim().to_owned();
        let teams_cnt = line.parse::<usize>().unwrap();
        let mut teams = Vec::default();
        loop {
            EnemyPool::generate_teams(teams_cnt, &mut teams, resources);
            println!("Enter indices to remove:");
            line.clear();
            std::io::stdin().read_line(&mut line).unwrap();
            line = line.trim().to_owned();
            if line.is_empty() {
                break;
            }
            let inds = line
                .split(' ')
                .map(|x| x.parse::<usize>().unwrap())
                .collect_vec();
            for ind in inds {
                teams.remove(ind);
            }
        }
        let mut ratings = Ratings::default();
        let mut cnt = 0;
        let save_path = static_path().join("levels.json");
        loop {
            Self::rate_teams(&teams, &mut ratings, 10, world, resources);
            cnt += 1;
            teams.sort_by(|a, b| {
                ratings
                    .get_rating(&a.name)
                    .total_cmp(&ratings.get_rating(&b.name))
            });
            let save = serde_json::to_string_pretty(&teams).unwrap();

            match std::fs::write(&save_path, save) {
                Ok(_) => debug!("Save levels to {:?}", &save_path),
                Err(error) => error!("Can't save levels: {}", error),
            }
            println!("\nRun#{cnt}{ratings}");
        }
    }

    pub fn rate_teams(
        teams: &Vec<PackedTeam>,
        ratings: &mut Ratings,
        match_count: usize,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let teams_str = teams
            .iter()
            .enumerate()
            .map(|(i, x)| format!("{}. {}", i, x.name.clone()))
            .join("\n");
        debug!("Start rating...\n{teams_str}");
        let mut cnt = 0.0;
        let total = (teams.len() * match_count) as f64;
        for light in teams.iter() {
            for _ in 0..match_count as i32 {
                let mut dark = light;
                while dark.name == light.name {
                    dark = teams.choose(&mut thread_rng()).unwrap();
                }
                debug!("Run battle {} x {}", light.name, dark.name);
                let result =
                    SimulationSystem::run_ranked_battle(light, dark, world, resources, None);
                debug!("Result: {result}");
                ratings.add_rating(&light.name, RatingType::WinRate, result, 3);
                cnt += 1.0;
            }
            debug!("{:.0}%", cnt / total * 100.0);
        }
        ratings.calculate();
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

    pub fn get_score(&self, name: &str, rating: RatingType) -> (usize, usize) {
        *self.data.get(name).unwrap().1.get(&rating).unwrap()
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
        let spaces = ".............................................";
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
