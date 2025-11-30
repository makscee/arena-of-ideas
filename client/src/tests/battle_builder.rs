use std::sync::OnceLock;

use super::*;

pub struct TestBuilder {
    teams: Vec<TeamBuilder>,
}

impl TestBuilder {
    pub fn new() -> Self {
        Self::init_test_resources();
        Self::init_test_logging();
        Self { teams: vec![] }
    }

    fn init_test_resources() {
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            crate::resources::init_for_tests();
            colorix().generate_palettes();
            init_style_map(&colorix());
        });
    }

    fn init_test_logging() {
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            struct TestLogger;

            impl log::Log for TestLogger {
                fn enabled(&self, metadata: &log::Metadata) -> bool {
                    metadata.level() <= log::Level::Debug
                }

                fn log(&self, record: &log::Record) {
                    if self.enabled(record.metadata()) {
                        println!(
                            "[{}] {} - {}",
                            record.level(),
                            record.target(),
                            record.args()
                        );
                    }
                }

                fn flush(&self) {}
            }

            log::set_boxed_logger(Box::new(TestLogger))
                .map(|()| log::set_max_level(log::LevelFilter::Debug))
                .ok();
        });
    }

    pub fn add_team(mut self, id: u64) -> Self {
        assert_eq!(id % 100, 0, "Team ID must be divisible by 100");
        self.teams.push(TeamBuilder::new(id));
        self
    }

    pub fn add_house(mut self, id: u64) -> Self {
        assert_eq!(id % 100, 0, "House ID must be divisible by 100");
        let team = self
            .teams
            .last_mut()
            .expect("Cannot add house without a team");
        team.add_house(id);
        self
    }

    pub fn add_unit(mut self, id: u64, pwr: i32, hp: i32) -> Self {
        assert_eq!(id % 100, 0, "Unit ID must be divisible by 100");
        let team = self
            .teams
            .last_mut()
            .expect("Cannot add unit without a team");
        team.add_unit(id, pwr, hp);
        self
    }

    pub fn add_reaction(mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) -> Self {
        let team = self
            .teams
            .last_mut()
            .expect("Cannot add reaction without a team");
        team.add_reaction(trigger, actions);
        self
    }

    pub fn add_ability(mut self, id: u64, actions: Vec<Action>) -> Self {
        let team = self
            .teams
            .last_mut()
            .expect("Cannot add ability without a team");
        team.add_ability(id, actions);
        self
    }

    pub fn add_status(
        mut self,
        id: u64,
        trigger: Trigger,
        actions: impl Into<Vec<Action>>,
    ) -> Self {
        let team = self
            .teams
            .last_mut()
            .expect("Cannot add status without a team");
        team.add_status(id, trigger, actions);
        self
    }

    pub fn run_battle(self) -> BattleTestResult {
        assert_eq!(self.teams.len(), 2, "Battle requires exactly 2 teams");

        let mut teams_built = vec![];
        for team_builder in self.teams {
            teams_built.push(team_builder.build());
        }

        let battle = Battle {
            id: 10000,
            left: teams_built[0].clone(),
            right: teams_built[1].clone(),
        };
        dbg!(&battle);

        BattleTestCase { battle }.run()
    }
}

struct TeamBuilder {
    id: u64,
    houses: Vec<HouseBuilder>,
}

impl TeamBuilder {
    fn new(id: u64) -> Self {
        Self { id, houses: vec![] }
    }

    fn add_house(&mut self, id: u64) {
        self.houses.push(HouseBuilder::new(id));
    }

    fn add_unit(&mut self, id: u64, pwr: i32, hp: i32) {
        let house = self
            .houses
            .last_mut()
            .expect("Cannot add unit without a house");
        house.add_unit(id, pwr, hp);
    }

    fn add_reaction(&mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) {
        let house = self
            .houses
            .last_mut()
            .expect("Cannot add reaction without a house");
        house.add_reaction(trigger, actions);
    }

    fn add_ability(&mut self, id: u64, actions: Vec<Action>) {
        let house = self
            .houses
            .last_mut()
            .expect("Cannot add ability without a house");
        house.add_ability(id, actions);
    }

    fn add_status(&mut self, id: u64, trigger: Trigger, actions: impl Into<Vec<Action>>) {
        let house = self
            .houses
            .last_mut()
            .expect("Cannot add status without a house");
        house.add_status(id, trigger, actions);
    }

    fn build(self) -> NTeam {
        let mut built_houses = vec![];
        let mut all_units = vec![];
        for house_builder in self.houses {
            let (house, units) = house_builder.build();
            all_units.extend(units);
            built_houses.push(house);
        }
        let mut slots = vec![];
        let id_offset = self.id * 10000;
        for (index, unit) in all_units.iter().enumerate() {
            let slot_id = id_offset + 3000 + (index as u64);
            let slot = NTeamSlot::new(slot_id, 0, index as i32).with_unit(unit.clone());
            slots.push(slot);
        }
        let mut team = NTeam::default();
        team.set_id(self.id);
        team.houses = OwnedMultiple::new_loaded(self.id, built_houses);
        team.slots = OwnedMultiple::new_loaded(self.id, slots);
        team
    }
}

pub struct HouseBuilder {
    pub id: u64,
    units: Vec<UnitBuilder>,
    ability: Option<AbilityBuilder>,
    status: Option<StatusBuilder>,
}

impl HouseBuilder {
    fn new(id: u64) -> Self {
        Self {
            id,
            units: vec![],
            ability: None,
            status: None,
        }
    }

    fn add_unit(&mut self, id: u64, pwr: i32, hp: i32) {
        self.units.push(UnitBuilder::new(id, pwr, hp));
    }

    fn add_reaction(&mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) {
        let unit = self
            .units
            .last_mut()
            .expect("Cannot add reaction without a unit");
        unit.reaction = Some(Reaction {
            trigger,
            actions: actions.into(),
        });
    }

    fn add_ability(&mut self, id: u64, actions: Vec<Action>) {
        self.ability = Some(AbilityBuilder::new(id, actions));
    }

    fn add_status(&mut self, id: u64, trigger: Trigger, actions: impl Into<Vec<Action>>) {
        self.status = Some(StatusBuilder::new(id, trigger, actions));
    }

    fn build(self) -> (NHouse, Vec<NUnit>) {
        let mut built_units = vec![];

        for unit_builder in self.units {
            built_units.push(unit_builder.build());
        }

        let color_id = 40000 + self.id;
        let mut color = NHouseColor::default();
        color.set_id(color_id);
        color.color = HexColor("#FF0000".to_string());

        let mut house = NHouse::default();
        house.set_id(self.id);
        house.house_name = format!("House {}", self.id);
        house.color = Component::new_loaded(color);
        house.units = RefMultiple::Ids {
            parent_id: self.id,
            node_ids: built_units.iter().map(|u| u.id).collect(),
        };

        if let Some(ability_builder) = self.ability {
            house.ability = Component::new_loaded(ability_builder.build());
        }

        if let Some(status_builder) = self.status {
            house.status = Component::new_loaded(status_builder.build());
        }

        (house, built_units)
    }
}

struct UnitBuilder {
    id: u64,
    pwr: i32,
    hp: i32,
    reaction: Option<Reaction>,
}

impl UnitBuilder {
    fn new(id: u64, pwr: i32, hp: i32) -> Self {
        Self {
            id,
            pwr,
            hp,
            reaction: None,
        }
    }

    fn build(self) -> NUnit {
        let mut stats = NUnitStats::default();
        stats.set_id(self.id + 1);
        stats.pwr = self.pwr;
        stats.hp = self.hp;

        let mut desc = NUnitDescription::default();
        desc.set_id(self.id + 2);
        desc.description = format!("Unit {}", self.id);

        let mut state = NUnitState::default();
        state.set_id(self.id + 3);
        state.stax = 1;

        let mut unit = NUnit::default();
        unit.set_id(self.id);
        unit.unit_name = format!("Unit {}", self.id);
        unit.stats = Component::new_loaded(stats);
        unit.description = Component::new_loaded(desc);
        unit.state = Component::new_loaded(state);
        if let Some(reaction) = self.reaction {
            let mut behavior = NUnitBehavior::default();
            behavior.set_id(self.id + 3);
            behavior.reactions = vec![reaction];
            unit.behavior = Component::new_loaded(behavior);
        }

        unit
    }
}

struct AbilityBuilder {
    id: u64,
    actions: Vec<Action>,
}

impl AbilityBuilder {
    fn new(id: u64, actions: Vec<Action>) -> Self {
        Self { id, actions }
    }

    fn build(self) -> NAbilityMagic {
        let desc_id = self.id + 1;
        let effect_id = self.id + 2;

        let mut effect = NAbilityEffect::default();
        effect.set_id(effect_id);
        effect.actions = self.actions;

        let mut desc = NAbilityDescription::default();
        desc.set_id(desc_id);
        desc.description = "Ability".to_string();

        let mut ability = NAbilityMagic::default();
        ability.set_id(self.id);
        ability.effect = Component::new_loaded(effect);
        ability.ability_name = format!("Ability {}", self.id);
        ability.description = Component::new_loaded(desc);

        ability
    }
}

struct StatusBuilder {
    id: u64,
    reactions: Vec<Reaction>,
}

impl StatusBuilder {
    fn new(id: u64, trigger: Trigger, actions: impl Into<Vec<Action>>) -> Self {
        Self {
            id,
            reactions: vec![Reaction {
                trigger,
                actions: actions.into(),
            }],
        }
    }

    fn build(self) -> NStatusMagic {
        let desc_id = self.id + 1;
        let behavior_id = self.id + 2;
        let rep_id = self.id + 3;

        let mut behavior = NStatusBehavior::default();
        behavior.set_id(behavior_id);
        behavior.reactions = self.reactions;

        let mut desc = NStatusDescription::default();
        desc.set_id(desc_id);
        desc.description = "Status".to_string();

        let mut representation = NStatusRepresentation::default();
        representation.set_id(rep_id);
        representation.material = Material::default();

        let mut state = NState::default();
        state.set_id(self.id + 4);
        state.stax = 1;

        let mut status = NStatusMagic::default();
        status.set_id(self.id);
        status.behavior = Component::new_loaded(behavior);
        status.status_name = format!("Status {}", self.id);
        status.description = Component::new_loaded(desc);
        status.representation = Component::new_loaded(representation);
        status.state = Component::new_loaded(state);

        status
    }
}

pub struct BattleTestCase {
    pub battle: Battle,
}

impl BattleTestCase {
    pub fn run(self) -> BattleTestResult {
        println!("=== STARTING BATTLE TEST ===");
        let mut source = self.battle.to_source();

        source
            .exec_context(|ctx| BattleSimulation::start(ctx))
            .unwrap();

        let max_iterations = 20;
        let mut iterations = 0;

        while iterations < max_iterations {
            let ended = source.battle().map(|s| s.ended()).unwrap_or(true);
            if ended {
                break;
            }

            println!("--- Battle iteration {} ---", iterations + 1);
            source
                .exec_context(|ctx| BattleSimulation::run(ctx))
                .unwrap();

            iterations += 1;
        }

        let (left_alive, right_alive, ended, log) = {
            let sim = source.battle().unwrap();
            (
                sim.left_units().len(),
                sim.right_units().len(),
                sim.ended(),
                sim.log.clone(),
            )
        };

        println!(
            "Left: {}, Right: {}, Ended: {}",
            left_alive, right_alive, ended
        );

        let winner = if ended {
            match (left_alive == 0, right_alive == 0) {
                (true, false) => Some(TeamSide::Right),
                (false, true) => Some(TeamSide::Left),
                _ => None,
            }
        } else {
            None
        };

        BattleTestResult {
            winner,
            left_alive,
            right_alive,
            iterations,
            log,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TeamSide {
    Left,
    Right,
}

pub struct BattleTestResult {
    pub winner: Option<TeamSide>,
    pub left_alive: usize,
    pub right_alive: usize,
    pub iterations: usize,
    pub log: BattleLog,
}

impl BattleTestResult {
    pub fn assert_winner(self, expected: TeamSide) -> Self {
        assert_eq!(
            self.winner,
            Some(expected),
            "Expected {:?} to win but got {:?}. Left: {}, Right: {}, Iterations: {}",
            expected,
            self.winner,
            self.left_alive,
            self.right_alive,
            self.iterations
        );
        self
    }

    pub fn assert_draw(self) -> Self {
        assert_eq!(
            self.winner, None,
            "Expected draw but {:?} won. Left: {}, Right: {}, Iterations: {}",
            self.winner, self.left_alive, self.right_alive, self.iterations
        );
        self
    }

    pub fn assert_units_alive(self, side: TeamSide, count: usize) -> Self {
        let actual = match side {
            TeamSide::Left => self.left_alive,
            TeamSide::Right => self.right_alive,
        };
        assert_eq!(
            actual, count,
            "Expected {} units alive on {:?} side but got {}",
            count, side, actual
        );
        self
    }

    pub fn assert_action_count(self, min: usize) -> Self {
        assert!(
            self.log.actions.len() >= min,
            "Expected at least {} actions but got {}",
            min,
            self.log.actions.len()
        );
        self
    }

    pub fn assert_iterations(self, count: usize) -> Self {
        assert_eq!(
            self.iterations, count,
            "Expected {} iterations but got {}",
            count, self.iterations
        );
        self
    }
}
