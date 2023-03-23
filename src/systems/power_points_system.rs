use super::*;

pub struct PowerPointsSystem {}

impl PowerPointsSystem {
    pub fn measure(
        pool: Vec<PackedUnit>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Vec<(PackedUnit, usize)> {
        let start = Instant::now();
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
        let duration = start.elapsed();
        debug!("Measured in: {:?}", duration);
        results
    }

    pub fn measure_single(
        index: usize,
        results: &mut Vec<(PackedUnit, usize)>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        fn choose_random(pool: &Vec<(PackedUnit, usize)>) -> &PackedUnit {
            &pool.iter().choose(&mut thread_rng()).unwrap().0
        }
        let mut result = 0;
        let unit = &results.get(index).unwrap().0;
        for _ in 0..8 {
            let light = vec![unit];
            let dark = vec![choose_random(results)];
            if SimulationSystem::run_battle(&light, &dark, world, resources, None) {
                result += 1;
            }
            let light = vec![unit, choose_random(results)];
            let dark = vec![choose_random(results), choose_random(results)];
            if SimulationSystem::run_battle(&light, &dark, world, resources, None) {
                result += 1;
            }
        }
        results.get_mut(index).unwrap().1 = result;
    }
}
