use std::sync::OnceLock;

use super::*;

pub struct TestBuilder {
    battle_builder: BattleBuilder,
}

impl TestBuilder {
    pub fn new() -> Self {
        Self::init_test_resources();
        Self::init_test_logging();
        Self {
            battle_builder: BattleBuilder::new(),
        }
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
        self.battle_builder = self.battle_builder.add_team(id);
        self
    }

    pub fn add_house(mut self, id: u64) -> Self {
        assert_eq!(id % 100, 0, "House ID must be divisible by 100");
        self.battle_builder = self.battle_builder.add_house(id);
        self
    }

    pub fn add_unit(mut self, id: u64, pwr: i32, hp: i32) -> Self {
        assert_eq!(id % 100, 0, "Unit ID must be divisible by 100");
        self.battle_builder = self.battle_builder.add_unit(id, pwr, hp);
        self
    }

    pub fn add_reaction(mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) -> Self {
        self.battle_builder = self.battle_builder.add_reaction(trigger, actions);
        self
    }

    pub fn add_ability(mut self, id: u64) -> Self {
        assert_eq!(id % 100, 0, "Ability ID must be divisible by 100");
        self.battle_builder = self.battle_builder.add_ability(id);
        self
    }

    pub fn add_ability_action(mut self, action: Action) -> Self {
        self.battle_builder = self.battle_builder.add_ability_action(action);
        self
    }

    pub fn add_status(mut self, id: u64) -> Self {
        assert_eq!(id % 100, 0, "Status ID must be divisible by 100");
        self.battle_builder = self.battle_builder.add_status(id);
        self
    }

    pub fn add_status_reaction(
        mut self,
        trigger: Trigger,
        actions: impl Into<Vec<Action>>,
    ) -> Self {
        self.battle_builder = self.battle_builder.add_status_reaction(trigger, actions);
        self
    }

    pub fn run_battle(self) -> BattleTestResult {
        self.battle_builder.run_battle()
    }
}

struct BattleBuilder {
    teams: Vec<TeamBuilder>,
}

impl BattleBuilder {
    fn new() -> Self {
        Self { teams: vec![] }
    }

    fn add_team(mut self, id: u64) -> Self {
        self.teams.push(TeamBuilder::new(id));
        self
    }

    fn add_house(mut self, id: u64) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_house(id);
        }
        self
    }

    fn add_unit(mut self, id: u64, pwr: i32, hp: i32) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_unit(id, pwr, hp);
        }
        self
    }

    fn add_reaction(mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_reaction(trigger, actions);
        }
        self
    }

    fn add_ability(mut self, id: u64) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_ability(id);
        }
        self
    }

    fn add_ability_action(mut self, action: Action) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_ability_action(action);
        }
        self
    }

    fn add_status(mut self, id: u64) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_status(id);
        }
        self
    }

    fn add_status_reaction(mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) -> Self {
        if let Some(team) = self.teams.last_mut() {
            team.add_status_reaction(trigger, actions);
        }
        self
    }

    fn run_battle(self) -> BattleTestResult {
        assert_eq!(self.teams.len(), 2, "Battle requires exactly 2 teams");

        let mut id_generator = IdGenerator::new();
        let mut teams_built = vec![];

        for team_builder in self.teams {
            teams_built.push(team_builder.build(&mut id_generator));
        }

        let battle = Battle {
            id: id_generator.next(),
            left: teams_built[0].clone(),
            right: teams_built[1].clone(),
        };

        BattleTestCase { battle }.run()
    }
}

struct IdGenerator {
    next_id: u64,
}

impl IdGenerator {
    fn new() -> Self {
        Self { next_id: 10000 }
    }

    fn next(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
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
        if let Some(house) = self.houses.last_mut() {
            house.add_unit(id, pwr, hp);
        }
    }

    fn add_reaction(&mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) {
        if let Some(house) = self.houses.last_mut() {
            if let Some(unit) = house.units.last_mut() {
                unit.reaction = Some(Reaction {
                    trigger,
                    actions: actions.into(),
                });
            }
        }
    }

    fn add_ability(&mut self, id: u64) {
        if let Some(house) = self.houses.last_mut() {
            house.ability = Some(AbilityBuilder::new(id));
        }
    }

    fn add_ability_action(&mut self, action: Action) {
        if let Some(house) = self.houses.last_mut() {
            if let Some(ability) = &mut house.ability {
                ability.actions.push(action);
            }
        }
    }

    fn add_status(&mut self, id: u64) {
        if let Some(house) = self.houses.last_mut() {
            house.status = Some(StatusBuilder::new(id));
        }
    }

    fn add_status_reaction(&mut self, trigger: Trigger, actions: impl Into<Vec<Action>>) {
        if let Some(house) = self.houses.last_mut() {
            if let Some(status) = &mut house.status {
                status.reactions.push(Reaction {
                    trigger,
                    actions: actions.into(),
                });
            }
        }
    }

    fn build(self, id_gen: &mut IdGenerator) -> NTeam {
        let mut built_houses = vec![];
        let mut all_units = vec![];

        for house_builder in self.houses {
            let (house, units) = house_builder.build(id_gen);
            all_units.extend(units);
            built_houses.push(house);
        }

        let mut fusions = vec![];
        for (index, unit) in all_units.iter().enumerate() {
            let fusion_id = id_gen.next();
            let slot_id = id_gen.next();

            let mut slot = NFusionSlot::default();
            slot.set_id(slot_id);
            slot.index = 0;
            slot.actions = UnitActionRange {
                start: 0,
                length: 255,
            };
            slot.unit = Ref::new_id(unit.id);

            let mut fusion = NFusion::default();
            fusion.set_id(fusion_id);
            fusion.index = index as i32;
            fusion.trigger_unit = Ref::Id(unit.id);
            fusion.actions_limit = 3;
            fusion.slots = OwnedMultiple::new_loaded(vec![slot]);

            fusions.push(fusion);
        }

        let mut team = NTeam::default();
        team.set_id(self.id);
        team.houses = OwnedMultiple::new_loaded(built_houses);
        team.fusions = OwnedMultiple::new_loaded(fusions);

        team
    }
}

struct HouseBuilder {
    id: u64,
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

    fn build(self, id_gen: &mut IdGenerator) -> (NHouse, Vec<NUnit>) {
        let color_id = id_gen.next();

        let mut color = NHouseColor::default();
        color.set_id(color_id);
        color.color = HexColor("#FF0000".to_string());

        let mut built_units = vec![];
        for unit_builder in self.units {
            built_units.push(unit_builder.build(id_gen));
        }

        let mut house = NHouse::default();
        house.set_id(self.id);
        house.house_name = format!("House {}", self.id);
        house.color = Component::new_loaded(color);
        house.units = OwnedMultiple::new_loaded(built_units.clone());

        if let Some(ability) = self.ability {
            house.ability = Component::new_loaded(ability.build(id_gen));
        }

        if let Some(status) = self.status {
            house.status = Component::new_loaded(status.build(id_gen));
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

    fn build(self, _id_gen: &mut IdGenerator) -> NUnit {
        let stats_id = self.id + 1;
        let desc_id = self.id + 2;
        let state_id = self.id + 3;

        let mut stats = NUnitStats::default();
        stats.set_id(stats_id);
        stats.pwr = self.pwr;
        stats.hp = self.hp;

        let mut desc = NUnitDescription::default();
        desc.set_id(desc_id);
        desc.description = format!("Unit {}", self.id);
        desc.trigger = Trigger::BattleStart;
        desc.magic_type = MagicType::Ability;

        if let Some(reaction) = self.reaction {
            let behavior_id = self.id + 4;
            let mut unit_behavior = NUnitBehavior::default();
            unit_behavior.set_id(behavior_id);
            unit_behavior.reaction = reaction;
            unit_behavior.magic_type = MagicType::Ability;
            desc.behavior = Component::new_loaded(unit_behavior);
        }

        let mut state = NState::default();
        state.set_id(state_id);
        state.stax = 1;

        let mut unit = NUnit::default();
        unit.set_id(self.id);
        unit.unit_name = format!("Unit {}", self.id);
        unit.stats = Component::new_loaded(stats);
        unit.description = Component::new_loaded(desc);
        unit.state = Component::new_loaded(state);

        unit
    }
}

struct AbilityBuilder {
    id: u64,
    actions: Vec<Action>,
}

impl AbilityBuilder {
    fn new(id: u64) -> Self {
        Self {
            id,
            actions: vec![],
        }
    }

    fn build(self, _id_gen: &mut IdGenerator) -> NAbilityMagic {
        let desc_id = self.id + 1;
        let effect_id = self.id + 2;

        let mut effect = NAbilityEffect::default();
        effect.set_id(effect_id);
        effect.actions = self.actions;

        let mut desc = NAbilityDescription::default();
        desc.set_id(desc_id);
        desc.description = "Ability".to_string();
        desc.effect = Component::new_loaded(effect);

        let mut ability = NAbilityMagic::default();
        ability.set_id(self.id);
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
    fn new(id: u64) -> Self {
        Self {
            id,
            reactions: vec![],
        }
    }

    fn build(self, _id_gen: &mut IdGenerator) -> NStatusMagic {
        let desc_id = self.id + 1;
        let behavior_id = self.id + 2;
        let rep_id = self.id + 3;

        let mut behavior = NStatusBehavior::default();
        behavior.set_id(behavior_id);
        behavior.reactions = self.reactions;

        let mut desc = NStatusDescription::default();
        desc.set_id(desc_id);
        desc.description = "Status".to_string();
        desc.behavior = Component::new_loaded(behavior);

        let mut representation = NStatusRepresentation::default();
        representation.set_id(rep_id);
        representation.material = Material::default();

        let mut status = NStatusMagic::default();
        status.set_id(self.id);
        status.status_name = format!("Status {}", self.id);
        status.description = Component::new_loaded(desc);
        status.representation = Component::new_loaded(representation);

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

pub fn new_unit(id: u64, pwr: i32, hp: i32) -> (u64, i32, i32) {
    (id, pwr, hp)
}

pub fn deal_3_dmg() -> (u64, Vec<Action>) {
    (
        1200,
        vec![
            Action::set_value(Box::new(Expression::i32(3))),
            Action::deal_damage,
        ],
    )
}

pub fn heal_1() -> (u64, Vec<Action>) {
    (
        1200,
        vec![
            Action::set_value(Box::new(Expression::i32(1))),
            Action::heal_damage,
        ],
    )
}
