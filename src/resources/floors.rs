use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Floors {
    #[serde(default)]
    current: usize,
    floors: Vec<Team>,
}

impl Floors {
    fn current(&self) -> &Team {
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
            .clone()
            .unpack_entries(&Faction::Dark, world, resources);
    }
}
