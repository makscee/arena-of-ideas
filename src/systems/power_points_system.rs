use super::*;

pub struct PowerPointsSystem {}

impl PowerPointsSystem {
    pub fn measure(
        pool: Vec<SerializedUnit>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Vec<(SerializedUnit, usize)> {
        if !resources
            .options
            .log
            .get(&LogContext::Measurement)
            .unwrap_or(&false)
        {
            resources.logger.set_enabled(false);
        }
        let mut results = pool.into_iter().map(|x| (x, 0)).collect_vec();
        for i in 0..results.len() {
            Self::measure_single(i, &mut results, world, resources);
        }
        resources.logger.set_enabled(true);
        results
    }

    pub fn measure_single(
        index: usize,
        results: &mut Vec<(SerializedUnit, usize)>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        fn choose_random(pool: &Vec<(SerializedUnit, usize)>) -> &SerializedUnit {
            &pool.iter().choose(&mut thread_rng()).unwrap().0
        }
        let mut result = 0;
        let unit = &results.get(index).unwrap().0;
        for _ in 0..8 {
            let light = vec![unit];
            let dark = vec![choose_random(results)];
            if Self::run_simulation(light, dark, world, resources) == Faction::Light {
                result += 1;
            }
            let light = vec![unit, choose_random(results)];
            let dark = vec![choose_random(results), choose_random(results)];
            if Self::run_simulation(light, dark, world, resources) == Faction::Light {
                result += 1;
            }
        }
        results.get_mut(index).unwrap().1 = result;
    }

    fn run_simulation(
        light: Vec<&SerializedUnit>,
        dark: Vec<&SerializedUnit>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Faction {
        UnitSystem::clear_factions(world, resources, &hashset! {Faction::Dark, Faction::Light});
        light.iter().enumerate().for_each(|(ind, unit)| {
            unit.unpack(world, resources, ind + 1, Faction::Light);
        });
        dark.iter().enumerate().for_each(|(ind, unit)| {
            unit.unpack(world, resources, ind + 1, Faction::Dark);
        });
        ActionSystem::run_ticks(world, resources);

        while let Some((left, right)) = BattleSystem::find_hitters(world) {
            BattleSystem::hit(left, right, world, resources);
            BattleSystem::death_check(world, resources);
            SlotSystem::fill_gaps(world, resources, hashset! {Faction::Light, Faction::Dark});
        }
        let result = match BattleSystem::battle_won(world) {
            true => Faction::Light,
            false => Faction::Dark,
        };
        resources.cassette.clear();
        UnitSystem::clear_factions(world, resources, &hashset! {Faction::Dark, Faction::Light});
        result
    }
}
