use super::*;

pub struct RatingSystem {}

impl RatingSystem {
    fn generate_team(size: usize, heroes: &Vec<PackedUnit>) -> PackedTeam {
        let mut units: Vec<PackedUnit> = default();
        for _ in 0..size {
            let unit = heroes.choose(&mut thread_rng()).unwrap().clone();
            units.push(unit);
        }
        PackedTeam::from_units(units)
    }

    pub fn simulate_hero_ratings_calculation(world: &mut legion::World, resources: &mut Resources) {
        resources.logger.set_enabled(false);

        let heroes = HeroPool::all(&resources);
        let mut teams: Vec<PackedTeam> = default();
        let mut team_ratings;

        // fill teams with random max size teams
        // each team X random matches against other
        // retain top 1/3
        // sum end place of each hero
        // fill 2/3 with mutations
        // repeat
        const TEAMS: usize = 80;
        const TOP: usize = 5;
        const SIZE: usize = 5;
        const MATCHES: usize = 20;
        for _ in 0..TEAMS {
            teams.push(Self::generate_team(SIZE, &heroes));
        }
        let mut top_heroes: HashMap<String, usize> =
            HashMap::from_iter(heroes.iter().map(|x| (x.name.clone(), 0)));
        loop {
            team_ratings = default();
            Self::rate_teams_random(&teams, &mut team_ratings, MATCHES, world, resources);
            println!("{team_ratings}");
            teams.sort_by(|a, b| {
                team_ratings
                    .get_rating(&a.name)
                    .total_cmp(&team_ratings.get_rating(&b.name))
            });
            teams.split_off(TOP).truncate(0);
            teams
                .iter()
                .map(|x| x.name.split(", ").collect_vec())
                .flatten()
                .unique()
                .for_each(|x| *top_heroes.entry(x.to_owned()).or_default() += 1);

            let teams_len = teams.len();
            for muts in 1..=2 {
                for i in 0..teams_len {
                    let mut team = teams[i].clone();
                    for _ in 0..muts {
                        Self::mutate(&mut team, &heroes);
                    }
                    teams.push(team);
                }
            }
            while teams.len() < TEAMS {
                teams.push(Self::generate_team(SIZE, &heroes));
            }
            let ratings_json = top_heroes
                .iter()
                .sorted_by_key(|x| x.1)
                .map(|(name, score)| format!("\"{name}\":{score}"))
                .join(",");
            println!("Ratings json:\n{{{ratings_json}}}");
            println!(
                "\nTop heroes:\n{}",
                top_heroes
                    .iter()
                    .sorted_by_key(|x| x.1)
                    .map(|(name, cnt)| format!("{cnt} {name}"))
                    .join("\n")
            );
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

    fn read_line() -> String {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line.trim().to_owned()
    }

    fn ask_remove_indices(teams: &mut Vec<PackedTeam>) -> bool {
        println!("\nEnter indices to remove:");
        let line = Self::read_line();
        if line.is_empty() {
            return false;
        }
        let inds = line
            .split(' ')
            .map(|x| x.parse::<usize>().unwrap())
            .collect_vec();
        for ind in inds.into_iter().sorted().rev() {
            println!("Remove team {}", &teams[ind].name);
            teams.remove(ind);
        }
        true
    }

    fn ask_y_no(q: &str) -> bool {
        println!("\n{q} [y]/n");
        Self::read_line() != "n"
    }

    fn print_teams(teams: &Vec<PackedTeam>) {
        println!(
            "Generated teams\n{}",
            teams
                .iter()
                .enumerate()
                .map(|(ind, team)| format!("{ind} {}", team.name))
                .join("\n")
        );
    }

    pub fn simulate_enemy_ratings_calculation(
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        resources.logger.set_enabled(false);
        const LEVELS: usize = 15;

        let mut teams = Ladder::all_teams(resources);
        if !teams.is_empty() {
            println!(
                "Current ladder:\n{}",
                teams.iter().map(|x| x.name.clone()).join("\n")
            );
            if Self::ask_y_no("Clear teams?") {
                teams.clear();
            }
        }
        loop {
            EnemyPool::fill_teams_vec(LEVELS * 3, &mut teams, resources);
            Self::print_teams(&teams);
            if !Self::ask_remove_indices(&mut teams) {
                break;
            }
        }
        let mut ratings = Ratings::default();
        let mut cnt = 0;
        loop {
            Self::rate_teams_full(&teams, &mut ratings, world, resources);
            cnt += 1;
            teams.sort_by(|a, b| {
                ratings
                    .get_rating(&a.name)
                    .total_cmp(&ratings.get_rating(&b.name))
            });
            Ladder::set_teams(&teams, resources);
            Ladder::save(resources);
            println!("\nRun#{cnt}{ratings}");
            if Self::ask_remove_indices(&mut teams) {
                ratings = Ratings::default();
                EnemyPool::fill_teams_vec(LEVELS * 3, &mut teams, resources);
            }
        }
    }

    pub fn rate_teams_full(
        teams: &Vec<PackedTeam>,
        ratings: &mut Ratings,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        for i in 0..teams.len() {
            for j in 0..teams.len() {
                if i == j {
                    continue;
                }
                let light = &teams[i];
                let dark = &teams[j];
                let result =
                    SimulationSystem::run_ranked_battle(light, dark, world, resources, None);
                ratings.add_rating(&light.name, RatingType::WinRate, result, 3);
            }
        }
        ratings.calculate();
    }

    pub fn rate_teams_random(
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
        let dots = ".......................................................................................................................................................";
        let mut result: String = default();
        let max_len = self.data.iter().map(|(name, _)| name.len()).max().unwrap();
        for (ind, (name, (score, ratings))) in self
            .data
            .iter()
            .sorted_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
            .enumerate()
        {
            let mut name = name.clone();
            name.push_str(dots);
            let (name, _) = name.split_at((max_len + 10).max(30));
            result += &format!(
                "\n{ind}. {name} {score} [{}]",
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
