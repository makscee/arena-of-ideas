use super::*;

mod archers;
mod assassins;
mod chainers;
mod charmers;
mod critters;
mod exploders;
mod freezers;
mod healers;
mod spawners;
mod splashers;
mod vampires;
mod warriors;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Alliance {
    Spawners,
    Assassins,
    Critters,
    Archers,
    Freezers,
    Warriors,
    Healers,
    Vampires,
    Exploders,
    Splashers,
    Chainers,
    Charmers,
}

impl Alliance {
    fn initialize(&self, logic: &mut Logic, party_members: usize) {
        match self {
            Alliance::Spawners => spawners::initialize(logic, party_members),
            Alliance::Assassins => assassins::initialize(logic, party_members),
            Alliance::Critters => critters::initialize(logic, party_members),
            Alliance::Archers => archers::initialize(logic, party_members),
            Alliance::Freezers => freezers::initialize(logic, party_members),
            Alliance::Warriors => warriors::initialize(logic, party_members),
            Alliance::Healers => healers::initialize(logic, party_members),
            Alliance::Vampires => vampires::initialize(logic, party_members),
            Alliance::Exploders => exploders::initialize(logic, party_members),
            Alliance::Splashers => splashers::initialize(logic, party_members),
            Alliance::Chainers => chainers::initialize(logic, party_members),
            Alliance::Charmers => charmers::initialize(logic, party_members),
        }
    }
}

impl Logic<'_> {
    pub fn initialize_alliances(&mut self, config: &Config) {
        for (alliance, count) in &config.alliances {
            alliance.initialize(self, *count);
        }
    }
}
