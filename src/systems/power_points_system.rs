use super::*;

pub struct PowerPointsSystem {}

impl PowerPointsSystem {
    pub fn measure(
        templates: &Vec<PathBuf>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> HashMap<PathBuf, usize> {
        let mut results = HashMap::from_iter(templates.iter().map(|path| (path.clone(), 0usize)));
        templates
            .iter()
            .for_each(|path| Self::measure_single(path, templates, world, resources, &mut results));
        dbg!(results
            .iter()
            .sorted_by_key(|(_, score)| score.clone())
            .rev()
            .collect_vec());
        results
    }

    pub fn measure_single(
        template: &PathBuf,
        all_templates: &Vec<PathBuf>,
        world: &mut legion::World,
        resources: &mut Resources,
        results: &mut HashMap<PathBuf, usize>,
    ) {
        fn choose_random(templates: &Vec<PathBuf>) -> &PathBuf {
            templates.choose(&mut thread_rng()).unwrap()
        }
        for _ in 0..3 {
            let light = vec![template];
            let dark = vec![choose_random(all_templates)];
            if Self::run_simulation(&light, &dark, world, resources) == Faction::Light {
                light
                    .iter()
                    .for_each(|path| *results.get_mut(*path).unwrap() += 1);
            } else {
                dark.iter()
                    .for_each(|path| *results.get_mut(*path).unwrap() += 1);
            }
            let light = vec![template, choose_random(all_templates)];
            let dark = vec![choose_random(all_templates), choose_random(all_templates)];
            if Self::run_simulation(&light, &dark, world, resources) == Faction::Light {
                light
                    .iter()
                    .for_each(|path| *results.get_mut(*path).unwrap() += 1);
            } else {
                dark.iter()
                    .for_each(|path| *results.get_mut(*path).unwrap() += 1);
            }
        }
    }

    fn run_simulation(
        light: &Vec<&PathBuf>,
        dark: &Vec<&PathBuf>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Faction {
        UnitSystem::clear_factions(world, resources, &hashset! {Faction::Dark, Faction::Light});
        light.iter().enumerate().for_each(|(ind, path)| {
            UnitTemplatesPool::create_unit_entity(
                path,
                resources,
                world,
                Faction::Light,
                ind + 1,
                vec2::ZERO,
            );
        });
        dark.iter().enumerate().for_each(|(ind, path)| {
            UnitTemplatesPool::create_unit_entity(
                path,
                resources,
                world,
                Faction::Dark,
                ind + 1,
                vec2::ZERO,
            );
        });
        ActionSystem::run_ticks(world, resources);

        while let Some((left, right)) = BattleSystem::find_hitters(world) {
            BattleSystem::hit(left, right, world, resources);
            BattleSystem::clear_dead(world, resources);
            SlotSystem::fill_gaps(world, hashset! {Faction::Light, Faction::Dark});
        }
        let result = match BattleSystem::battle_won(world) {
            true => Faction::Light,
            false => Faction::Dark,
        };
        UnitSystem::clear_factions(world, resources, &hashset! {Faction::Dark, Faction::Light});
        result
    }
}
