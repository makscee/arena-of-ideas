use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Floors {
    #[serde(default)]
    current: usize,
    floors: Vec<Floor>,
}

impl Floors {
    fn current(&self) -> &Floor {
        &self.floors[self.current]
    }

    pub fn current_ind(&self) -> usize {
        self.current
    }

    pub fn reset(&mut self) {
        self.current = default();
    }

    pub fn next(&mut self) -> bool {
        self.current += 1;
        self.current < self.floors.len()
    }

    pub fn load(world: &mut legion::World, resources: &mut Resources) {
        resources
            .floors
            .current()
            .enemies
            .clone()
            .iter()
            .enumerate()
            .for_each(|(slot, path)| {
                UnitTemplatesPool::create_unit_entity(
                    &static_path().join(path),
                    resources,
                    world,
                    Faction::Dark,
                    slot,
                    vec2::ZERO,
                );
            })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Floor {
    pub enemies: Vec<PathBuf>,
}
