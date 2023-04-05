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
            let (floor, team) = Self::run_single(world, resources);
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
                .map(|x| x.state.name.clone())
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

    fn run_single(world: &mut legion::World, resources: &mut Resources) -> (usize, Team) {
        let pool: HashMap<String, PackedUnit> = HashMap::from_iter(
            resources
                .hero_pool
                .all()
                .into_iter()
                .map(|x| (x.name.clone(), x)),
        );

        let mut team = Team::new("light".to_string(), vec![]);
        ShopSystem::init_game(world, resources);
        let buy_price = ShopSystem::buy_price(resources);
        let sell_price = ShopSystem::sell_price(resources);
        const MAX_ARRANGE_TRIES: usize = 5;
        loop {
            let extra_units = {
                let mut value = ShopSystem::floor_money(resources.floors.current_ind()) / buy_price;
                value += value * sell_price / buy_price;
                value
            } as usize;
            let dark = resources.floors.current().clone();

            let shop_case = pool
                .values()
                .choose_multiple(&mut thread_rng(), SLOTS_COUNT * extra_units);
            let mut battle_result = false;
            for _ in 0..MAX_ARRANGE_TRIES {
                let mut new_units = vec![];
                for _ in 0..extra_units {
                    new_units.push(*shop_case.choose(&mut thread_rng()).unwrap());
                }
                team.unpack(&Faction::Team, world, resources);
                Event::ShopEnd.send(world, resources);
                Event::ShopStart.send(world, resources);
                let slots = (1..=SLOTS_COUNT).choose_multiple(&mut thread_rng(), extra_units);
                for (i, unit) in new_units.into_iter().enumerate() {
                    let slot = *slots.get(i).unwrap();
                    let entity = unit.unpack(world, resources, slot, Faction::Shop, None);
                    if team.units.len() + i < SLOTS_COUNT {
                        SlotSystem::make_gap(world, resources, slot, &hashset! {Faction::Team});
                    } else {
                        if let Some(entity) =
                            SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
                        {
                            ShopSystem::sell(entity, resources, world);
                        }
                    }
                    ShopSystem::buy(entity, slot, resources, world, &mut None);
                    ActionSystem::run_ticks(world, resources, &mut None);
                }
                let new_team = Team::pack(&Faction::Team, world, resources);
                UnitSystem::clear_faction(world, resources, Faction::Team);
                battle_result =
                    SimulationSystem::run_battle(&new_team, &dark, world, resources, None);
                resources.action_queue.clear();
                if battle_result {
                    team = new_team;
                    break;
                }
            }
            if !battle_result || !resources.floors.next() {
                break;
            }
        }
        let floor_reached = resources.floors.current_ind();
        resources.floors.reset();
        (floor_reached, team)
    }
}
