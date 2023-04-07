use super::*;

pub struct WalkthroughSystem {}

impl WalkthroughSystem {
    pub fn run_simulation(world: &mut legion::World, resources: &mut Resources) {
        resources.logger.set_enabled(false);
        let mut avg_levels: HashMap<String, (usize, usize)> = default();
        let mut floors: Vec<usize> = vec![0; resources.ladder.count() + 1];
        let mut i: usize = 0;
        let mut total_pick_data: HashMap<String, (usize, usize)> = default();
        loop {
            i += 1;
            let run_timer = Instant::now();
            let (floor, team, pick_data) = Self::run_single(world, resources);
            *floors.get_mut(floor).unwrap() += 1;
            for unit in team.units.iter() {
                let mut result = avg_levels.remove(&unit.name).unwrap_or_default();
                result.0 += floor;
                result.1 += 1;
                avg_levels.insert(unit.name.clone(), result);
            }
            for (name, data) in pick_data {
                let mut total_data = total_pick_data.remove(&name).unwrap_or_default();
                total_data.0 += data.0;
                total_data.1 += data.1;
                total_pick_data.insert(name, total_data);
            }

            let pick_rates = total_pick_data
                .iter()
                .map(|(name, (pick, show))| (name, *pick as f32 / *show as f32, *pick, *show))
                .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .collect_vec();

            println!("\nAvg level reached:");
            let mut avg_levels = avg_levels
                .iter()
                .map(|(name, (floors, games))| (name, *floors as f32 / *games as f32))
                .collect_vec();
            avg_levels.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            println!(
                "{}",
                avg_levels
                    .iter()
                    .map(|(name, avg_flr)| format!("\"{}\": {:.3}", name, avg_flr))
                    .join(",\n"),
            );
            println!("\nPick rates:");
            println!(
                "{}",
                pick_rates
                    .iter()
                    .map(|(name, rate, pick, show)| format!(
                        "\"{}\": {:.3} {}/{}",
                        name, rate, pick, show
                    ))
                    .join(",\n"),
            );
            let mut sorting: HashMap<String, f32> = HashMap::from_iter(
                pick_rates
                    .iter()
                    .enumerate()
                    .map(|(ind, (name, _, _, _))| (name.deref().clone(), ind as f32)),
            );
            for (i, (name, _)) in avg_levels.into_iter().enumerate() {
                let mut data = sorting.remove(name).unwrap_or_default();
                data = (data + i as f32) * 0.5;
                sorting.insert(name.clone(), data);
            }

            println!("\nResult:");
            println!(
                "{}",
                sorting
                    .iter()
                    .sorted_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(name, score)| format!("\"{}\": {:.1}", name, score))
                    .join(",\n"),
            );

            for (i, name) in resources
                .ladder
                .teams
                .iter()
                .map(|x| x.team.state.name.clone())
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
    ) -> (usize, Team, HashMap<String, (usize, usize)>) {
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
        let mut pick_show_count = HashMap::default();
        loop {
            let extra_units = {
                let mut value = ShopSystem::floor_money(resources.ladder.current_ind()) / buy_price;
                value += value * sell_price / buy_price;
                value
            } as usize;
            let dark = Ladder::generate_team(resources);

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
                team.unpack(&Faction::Team, world, resources);
                Event::ShopEnd.send(world, resources);
                Event::ShopStart.send(world, resources);
                let slots = (1..=MAX_SLOTS).choose_multiple(&mut thread_rng(), extra_units);
                for (i, unit) in new_units.iter().enumerate() {
                    let slot = *slots.get(i).unwrap();
                    let entity = unit.unpack(world, resources, slot, Faction::Shop, None);
                    if team.units.len() + i < MAX_SLOTS {
                        SlotSystem::make_gap(world, resources, slot, &hashset! {Faction::Team});
                    } else {
                        if let Some(entity) =
                            SlotSystem::find_unit_by_slot(slot, &Faction::Team, world, resources)
                        {
                            ShopSystem::sell(entity, resources, world);
                        }
                    }
                    ShopSystem::buy(entity, slot, resources, world, &mut None);
                    ActionSystem::run_ticks(world, resources, &mut None);
                }
                let new_team = Team::pack(&Faction::Team, world, resources);
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
            }
        }
        let level_reached = resources.ladder.current_ind();
        resources.ladder.reset();
        (level_reached, team, pick_show_count)
    }
}
