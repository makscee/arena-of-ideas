use super::*;

pub struct SimulationSystem {}

impl SimulationSystem {
    pub fn run_battle(
        light: &Vec<&PackedUnit>,
        dark: &Vec<&PackedUnit>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> bool {
        light.iter().enumerate().for_each(|(slot, unit)| {
            unit.unpack(world, resources, slot + 1, Faction::Light, None);
        });
        dark.iter().enumerate().for_each(|(slot, unit)| {
            unit.unpack(world, resources, slot + 1, Faction::Dark, None);
        });
        ActionSystem::run_ticks(world, resources);
        let mut ticks = 0;
        while let Some((left, right)) = BattleSystem::find_hitters(world) {
            ticks += 1;
            BattleSystem::hit(left, right, &mut None, world, resources);
            BattleSystem::death_check(world, resources);
            SlotSystem::fill_gaps(world, resources, &hashset! {Faction::Light, Faction::Dark});
            if ticks > 1000 {
                panic!("Exceeded ticks limit")
            }
        }
        let result = BattleSystem::battle_won(world);
        BattleSystem::clear_world(world, resources);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (legion::World, Resources) {
        let mut world = legion::World::default();
        let mut resources = Resources::new(Options::load());
        Game::init_world(&mut resources, &mut world);
        (world, resources)
    }

    #[test]
    fn test_simple() {
        setup();
        let (mut world, mut resources) = setup();
        let unit = PackedUnit {
            name: "test".to_string(),
            description: "test".to_string(),
            health: 1,
            damage: 0,
            attack: 1,
            houses: default(),
            trigger: default(),
            active_statuses: default(),
            shader: default(),
        };
        let light = vec![&unit];
        let dark = vec![&unit];
        assert!(SimulationSystem::run_battle(
            &light,
            &dark,
            &mut world,
            &mut resources
        ))
    }
}
