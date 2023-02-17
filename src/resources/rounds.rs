use super::*;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Rounds {
    #[serde(default)]
    pub next_round: usize,
    rounds: Vec<Round>,
}

impl Rounds {
    pub fn new() -> Self {
        Self {
            next_round: default(),
            rounds: default(),
        }
    }

    fn next(&mut self) -> &Round {
        let round = &self.rounds[self.next_round];
        self.next_round += 1;
        round
    }

    pub fn load(world: &mut legion::World, resources: &mut Resources) {
        resources
            .rounds
            .next()
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
