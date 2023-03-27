use super::*;

pub struct WalkthroughSystem {}

impl WalkthroughSystem {
    pub fn run_simulation(world: &mut legion::World, resources: &mut Resources) {
        resources.logger.set_enabled(false);
        let mut results: HashMap<String, (usize, usize)> = default();
        let mut floors: Vec<usize> = vec![0; resources.floors.count()];
        for _ in 0..3 {
            let (floor, run_results) = Self::run_single(world, resources);
            *floors.get_mut(floor).unwrap() += 1;
            for (key, (games, wins)) in run_results {
                let mut result = results.remove(&key).unwrap_or_default();
                result.0 += games;
                result.1 += wins;
                results.insert(key, result);
            }
        }
        let mut results = results
            .iter()
            .map(|(name, (games, wins))| (name, *wins as f32 / *games as f32))
            .collect_vec();
        results.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        dbg!(results);

        for (i, team) in resources.floors.teams.iter().enumerate() {
            debug!("{} {} = {}", i, team.name, floors.get(i).unwrap());
        }
    }

    fn run_single(
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> (usize, HashMap<String, (usize, usize)>) {
        let pool: HashMap<String, PackedUnit> = HashMap::from_iter(
            resources
                .hero_pool
                .all()
                .into_iter()
                .map(|x| (x.name.clone(), x)),
        );
        let mut results: HashMap<String, (usize, usize)> = default();

        let mut team = Team::new("light".to_string(), vec![]);
        let mut lost = false;
        loop {
            let extra_units = ShopSystem::floor_money(resources.floors.current_ind()) / UNIT_COST;
            let dark = resources.floors.current().clone();
            for _ in 0..extra_units {
                let shop_case = pool
                    .values()
                    .choose_multiple(&mut thread_rng(), SLOTS_COUNT);
                let mut winners: Vec<Team> = default();
                for shop_unit in shop_case {
                    for slot in 1..=SLOTS_COUNT {
                        team.unpack(&Faction::Team, world, resources);
                        let entity = shop_unit.unpack(world, resources, slot, Faction::Shop, None);
                        if let Some((entity, _)) =
                            SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
                        {
                            ShopSystem::sell(entity, resources, world);
                        }
                        ShopSystem::buy(entity, slot, resources, world, &mut None);
                        let mut result = results.remove(&shop_unit.name).unwrap_or_default();
                        result.0 += 1;
                        let team = Team::pack(&Faction::Team, world, resources);
                        let timer = Instant::now();
                        let battle_result =
                            SimulationSystem::run_battle(&team, &dark, world, resources, None);
                        debug!(
                            "{:.0}ms {} {} {}",
                            timer.elapsed().as_secs_f64() * 1000.0,
                            battle_result,
                            team,
                            dark
                        );
                        if battle_result {
                            winners.push(team);
                            result.1 += 1;
                        }
                        results.insert(shop_unit.name.clone(), result);
                        UnitSystem::clear_faction(world, resources, Faction::Team);
                    }
                }
                if winners.is_empty() {
                    lost = true;
                    break;
                }
                team = winners.into_iter().choose(&mut thread_rng()).unwrap();
            }
            if lost || !resources.floors.next() {
                break;
            }
        }
        let floor_reached = resources.floors.current_ind();
        resources.floors.reset();
        (floor_reached, results)
    }
}
