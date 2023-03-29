use super::*;

pub struct WalkthroughSystem {}

impl WalkthroughSystem {
    pub fn run_simulation(world: &mut legion::World, resources: &mut Resources) {
        resources.logger.set_enabled(false);
        let mut results: HashMap<String, (usize, usize)> = default();
        let mut floors: Vec<usize> = vec![0; resources.floors.count() + 1];
        let mut i: usize = 0;
        loop {
            i += 1;
            let run_timer = Instant::now();
            let (floor, run_results, team) = Self::run_single(world, resources);
            *floors.get_mut(floor).unwrap() += 1;
            for unit in team.units.iter() {
                let mut result = results.remove(&unit.name).unwrap_or_default();
                result.0 += floor;
                result.1 += 1;
                results.insert(unit.name.clone(), result);
            }
            let mut results = results
                .iter()
                .map(|(name, (floors, games))| (name, *floors as f32 / *games as f32))
                .collect_vec();
            results.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            println!(
                "{}",
                results
                    .into_iter()
                    .map(|(name, avg_flr)| format!("\"{}\": {:.3}", name, avg_flr))
                    .join(",\n"),
            );

            for (i, name) in resources
                .floors
                .teams
                .iter()
                .map(|x| x.name.clone())
                .chain(Some("Game Over".to_string()))
                .enumerate()
            {
                println!("{} {} = {}", i, name, floors.get(i).unwrap());
            }
            println!(
                "Run #{} took {:?} reached {} {}",
                i,
                run_timer.elapsed(),
                floor,
                team
            );
        }
    }

    fn run_single(
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> (usize, HashMap<String, (usize, usize)>, Team) {
        let pool: HashMap<String, PackedUnit> = HashMap::from_iter(
            resources
                .hero_pool
                .all()
                .into_iter()
                .map(|x| (x.name.clone(), x)),
        );
        let mut win_stats: HashMap<String, (usize, usize)> = default();

        let mut team = Team::new("light".to_string(), vec![]);
        let mut lost = false;
        loop {
            let extra_units = {
                let mut value = ShopSystem::floor_money(resources.floors.current_ind()) / UNIT_COST;
                value += (value >= 3) as usize;
                value
            };
            let dark = resources.floors.current().clone();
            // println!("{} start...", dark.name);
            let floor_timer = Instant::now();
            for _ in 0..extra_units {
                let shop_case = pool
                    .values()
                    .choose_multiple(&mut thread_rng(), SLOTS_COUNT);
                let mut winners: Vec<Team> = default();
                for shop_unit in shop_case {
                    for slot in 1..=SLOTS_COUNT {
                        team.unpack(&Faction::Team, world, resources);
                        Event::FloorEnd.send(world, resources);
                        let entity = shop_unit.unpack(world, resources, slot, Faction::Shop, None);
                        if team.units.len() < SLOTS_COUNT {
                            SlotSystem::make_gap(world, resources, slot, &hashset! {Faction::Team});
                        } else {
                            if let Some((entity, _)) =
                                SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
                            {
                                ShopSystem::sell(entity, resources, world);
                            }
                        }
                        ShopSystem::buy(entity, slot, resources, world, &mut None);
                        let team = Team::pack(&Faction::Team, world, resources);
                        let timer = Instant::now();
                        let battle_result =
                            SimulationSystem::run_battle(&team, &dark, world, resources, None);
                        // debug!(
                        //     "{:.1} ms {} {} {}",
                        //     timer.elapsed().as_secs_f64() * 1000.0,
                        //     battle_result,
                        //     team,
                        //     dark
                        // );
                        // for unit in team.units.iter() {
                        {
                            let name = shop_unit.name.clone();
                            let mut stats = win_stats.remove(&name).unwrap_or_default();
                            stats.1 += 1;
                            if battle_result {
                                stats.0 += 1;
                            }
                            win_stats.insert(name.clone(), stats);
                        }
                        if battle_result {
                            let team = Team::pack(&Faction::Team, world, resources);
                            winners.push(team);
                        }
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
            // println!("Win. {:?}", floor_timer.elapsed());
        }
        let floor_reached = resources.floors.current_ind();
        resources.floors.reset();
        (floor_reached, win_stats, team)
    }
}
