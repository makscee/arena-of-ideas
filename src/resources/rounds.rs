use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Rounds {
    #[serde(default)]
    next_round: usize,
    rounds: Vec<Round>,
}

pub const ROUNDS_COUNT: usize = 10;

impl Rounds {
    fn current(&self) -> &Round {
        &self.rounds[self.next_round - 1]
    }

    pub fn current_ind(&self) -> usize {
        self.next_round
    }

    pub fn reset(&mut self) {
        self.next_round = default();
    }

    pub fn next(&mut self) {
        self.next_round += 1;
    }

    pub fn load(world: &mut legion::World, resources: &mut Resources) {
        resources
            .rounds
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
pub struct Round {
    pub enemies: Vec<PathBuf>,
}
