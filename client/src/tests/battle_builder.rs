use std::sync::OnceLock;

use super::*;

pub struct TestBuilder {
    next_id: u64,
}

impl TestBuilder {
    pub fn new() -> Self {
        Self::init_test_resources();
        Self::init_test_logging();
        Self { next_id: 1000 }
    }

    fn init_test_resources() {
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            crate::resources::init_for_tests();
            colorix().generate_palettes();
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

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn create_unit(&mut self, name: &str, pwr: i32, hp: i32) -> UnitBuilder {
        UnitBuilder::new(name, pwr, hp)
    }

    pub fn create_unit_with_behavior(
        &mut self,
        name: &str,
        pwr: i32,
        hp: i32,
        behavior: Reaction,
    ) -> UnitBuilder {
        UnitBuilder::new(name, pwr, hp).behavior(behavior)
    }

    pub fn create_house(&mut self, name: &str, color: &str) -> HouseBuilder {
        HouseBuilder::new(name, color)
    }

    pub fn create_status(&mut self, name: &str) -> StatusBuilder {
        StatusBuilder::new(name)
    }

    pub fn create_ability(&mut self, name: &str) -> AbilityBuilder {
        AbilityBuilder::new(name)
    }

    pub fn create_team(&mut self) -> TeamBuilder {
        TeamBuilder::new()
    }

    pub fn create_battle(
        &mut self,
        left_team_builder: TeamBuilder,
        right_team_builder: TeamBuilder,
    ) -> BattleTestCase {
        let left_team = left_team_builder.build(self);
        let right_team = right_team_builder.build(self);

        let battle = Battle {
            id: self.next_id(),
            left: left_team,
            right: right_team,
        };
        BattleTestCase { battle }
    }

    pub fn create_simple_house(
        &mut self,
        name: &str,
        unit_builders: Vec<UnitBuilder>,
    ) -> HouseBuilder {
        let mut house = self.create_house(name, "#FF0000");
        for unit_builder in unit_builders {
            house = house.add_unit(unit_builder);
        }
        house
    }
}

pub struct UnitBuilder {
    name: String,
    pwr: i32,
    hp: i32,
    trigger: Trigger,
    magic_type: MagicType,
    behavior: Option<Reaction>,
    description: String,
}

impl UnitBuilder {
    pub fn new(name: &str, pwr: i32, hp: i32) -> Self {
        Self {
            name: name.to_string(),
            pwr,
            hp,
            trigger: Trigger::BattleStart,
            magic_type: MagicType::Ability,
            behavior: None,
            description: "Test unit".to_string(),
        }
    }

    pub fn trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn magic_type(mut self, magic_type: MagicType) -> Self {
        self.magic_type = magic_type;
        self
    }

    pub fn behavior(mut self, behavior: Reaction) -> Self {
        self.behavior = Some(behavior);
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn build(self, builder: &mut TestBuilder) -> NUnit {
        let unit_id = builder.next_id();
        let stats_id = builder.next_id();
        let desc_id = builder.next_id();
        let state_id = builder.next_id();

        let mut stats = NUnitStats::default();
        stats.set_id(stats_id);
        stats.pwr = self.pwr;
        stats.hp = self.hp;

        let mut desc = NUnitDescription::default();
        desc.set_id(desc_id);
        desc.description = self.description;
        desc.trigger = self.trigger;
        desc.magic_type = self.magic_type;

        if let Some(behavior) = self.behavior {
            let behavior_id = builder.next_id();
            let mut unit_behavior = NUnitBehavior::default();
            unit_behavior.set_id(behavior_id);
            unit_behavior.reaction = behavior;
            unit_behavior.magic_type = self.magic_type;
            desc.behavior = Component::new_loaded(unit_behavior);
        }

        let mut state = NUnitState::default();
        state.set_id(state_id);
        state.stacks = 1;

        let mut unit = NUnit::default();
        unit.set_id(unit_id);
        unit.unit_name = self.name;
        unit.stats = Component::new_loaded(stats);
        unit.description = Component::new_loaded(desc);
        unit.state = Component::new_loaded(state);

        unit
    }
}

pub struct HouseBuilder {
    name: String,
    color: String,
    units: Vec<UnitBuilder>,
    ability: Option<AbilityBuilder>,
    status: Option<StatusBuilder>,
}

impl HouseBuilder {
    pub fn new(name: &str, color: &str) -> Self {
        Self {
            name: name.to_string(),
            color: color.to_string(),
            units: vec![],
            ability: None,
            status: None,
        }
    }

    pub fn add_unit(mut self, unit: UnitBuilder) -> Self {
        self.units.push(unit);
        self
    }

    pub fn ability(mut self, ability: AbilityBuilder) -> Self {
        self.ability = Some(ability);
        self
    }

    pub fn status(mut self, status: StatusBuilder) -> Self {
        self.status = Some(status);
        self
    }

    pub fn build(self, builder: &mut TestBuilder) -> NHouse {
        let house_id = builder.next_id();
        let color_id = builder.next_id();

        let mut color = NHouseColor::default();
        color.set_id(color_id);
        color.color = HexColor::from(self.color);

        let built_units: Vec<NUnit> = self
            .units
            .into_iter()
            .map(|unit| unit.build(builder))
            .collect();

        let mut house = NHouse::default();
        house.set_id(house_id);
        house.house_name = self.name;
        house.color = Component::new_loaded(color);
        house.units = OwnedMultiple::new_loaded(built_units);

        if let Some(ability) = self.ability {
            house.ability = Component::new_loaded(ability.build(builder));
        }

        if let Some(status) = self.status {
            house.status = Component::new_loaded(status.build(builder));
        }

        house
    }
}

pub struct AbilityBuilder {
    name: String,
    description: String,
    actions: Vec<Action>,
}

impl AbilityBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "Test ability".to_string(),
            actions: vec![],
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn add_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn deal_3_damage(self) -> Self {
        self.add_action(Action::set_value(Expression::i32(3).into()))
            .add_action(Action::deal_damage)
    }

    pub fn heal(self) -> Self {
        self.add_action(Action::set_value(Box::new(Expression::i32(1))))
            .add_action(Action::heal_damage)
    }

    pub fn build(self, builder: &mut TestBuilder) -> NAbilityMagic {
        let ability_id = builder.next_id();
        let desc_id = builder.next_id();
        let effect_id = builder.next_id();

        let mut effect = NAbilityEffect::default();
        effect.set_id(effect_id);
        effect.actions = self.actions;

        let mut desc = NAbilityDescription::default();
        desc.set_id(desc_id);
        desc.description = self.description;
        desc.effect = Component::new_loaded(effect);

        let mut ability = NAbilityMagic::default();
        ability.set_id(ability_id);
        ability.ability_name = self.name;
        ability.description = Component::new_loaded(desc);

        ability
    }
}

pub struct StatusBuilder {
    name: String,
    description: String,
    reactions: Vec<Reaction>,
}

impl StatusBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: "Test status".to_string(),
            reactions: vec![],
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn add_reaction(mut self, trigger: Trigger, actions: Vec<Action>) -> Self {
        self.reactions.push(Reaction { trigger, actions });
        self
    }

    pub fn build(self, builder: &mut TestBuilder) -> NStatusMagic {
        let status_id = builder.next_id();
        let desc_id = builder.next_id();
        let behavior_id = builder.next_id();
        let rep_id = builder.next_id();

        let mut behavior = NStatusBehavior::default();
        behavior.set_id(behavior_id);
        behavior.reactions = self.reactions;

        let mut desc = NStatusDescription::default();
        desc.set_id(desc_id);
        desc.description = self.description;
        desc.behavior = Component::new_loaded(behavior);

        let mut representation = NStatusRepresentation::default();
        representation.set_id(rep_id);
        representation.material = Material::default();

        let mut status = NStatusMagic::default();
        status.set_id(status_id);
        status.status_name = self.name;
        status.description = Component::new_loaded(desc);
        status.representation = Component::new_loaded(representation);

        status
    }
}

pub struct FusionBuilder {
    unit_indices: Vec<usize>,
    trigger_index: usize,
}

impl FusionBuilder {
    pub fn new(unit_indices: Vec<usize>) -> Self {
        Self {
            unit_indices,
            trigger_index: 0,
        }
    }

    pub fn single(unit_index: usize) -> Self {
        Self::new(vec![unit_index])
    }

    pub fn trigger_unit(mut self, index: usize) -> Self {
        self.trigger_index = index;
        self
    }

    pub fn build(self, builder: &mut TestBuilder, house_units: &[NUnit]) -> NFusion {
        let fusion_id = builder.next_id();

        let mut slots = vec![];

        for (slot_index, &unit_index) in self.unit_indices.iter().enumerate() {
            let unit = &house_units[unit_index];

            let slot_id = builder.next_id();
            let mut slot = NFusionSlot::default();
            slot.set_id(slot_id);
            slot.index = slot_index as i32;
            slot.actions = UnitActionRange {
                trigger: self.trigger_index as u8,
                start: 0,
                length: 255,
            };
            slot.unit = Ref::new_id(unit.id);
            slots.push(slot);
        }

        let mut fusion = NFusion::default();
        fusion.set_id(fusion_id);
        fusion.index = 0;
        fusion.trigger_unit = house_units[self.trigger_index].id;
        fusion.actions_limit = 3;
        fusion.slots = OwnedMultiple::new_loaded(slots);

        fusion
    }
}

pub struct TeamBuilder {
    houses: Vec<HouseBuilder>,
    fusions: Vec<FusionBuilder>,
}

impl TeamBuilder {
    pub fn new() -> Self {
        Self {
            houses: vec![],
            fusions: vec![],
        }
    }

    pub fn add_house(mut self, house: HouseBuilder) -> Self {
        self.houses.push(house);
        self
    }

    pub fn add_fusion(mut self, fusion: FusionBuilder) -> Self {
        self.fusions.push(fusion);
        self
    }

    pub fn build(self, builder: &mut TestBuilder) -> NTeam {
        let team_id = builder.next_id();

        // Build houses first to get all units
        let built_houses: Vec<NHouse> = self
            .houses
            .into_iter()
            .map(|house| house.build(builder))
            .collect();

        // Collect all units from all houses
        let mut all_units = vec![];
        for house in &built_houses {
            if let Ok(units) = house.units.get() {
                all_units.extend(units.iter().cloned());
            }
        }

        // Build fusions that reference the house units
        let mut built_fusions = vec![];
        for (index, fusion_builder) in self.fusions.into_iter().enumerate() {
            let mut fusion = fusion_builder.build(builder, &all_units);
            fusion.index = index as i32;
            built_fusions.push(fusion);
        }

        let mut team = NTeam::default();
        team.set_id(team_id);
        team.houses = OwnedMultiple::new_loaded(built_houses);
        team.fusions = OwnedMultiple::new_loaded(built_fusions);

        team
    }
}

pub struct BattleTestCase {
    pub battle: Battle,
}

impl BattleTestCase {
    pub fn run(self) -> BattleTestResult {
        println!("=== STARTING BATTLE TEST ===");
        let mut source = self.battle.to_source();

        // Create context once and reuse it
        let mut context = source.as_context();

        // Start the simulation
        BattleSimulation::start(&mut context).unwrap();

        let max_iterations = 10; // Reduce iterations for focused testing
        let mut iterations = 0;

        while iterations < max_iterations {
            let ended = context.battle().map(|s| s.ended()).unwrap_or(true);
            if ended {
                break;
            }

            println!("--- Battle iteration {} ---", iterations + 1);

            BattleSimulation::run(&mut context).unwrap();

            iterations += 1;
        }

        let (left_alive, right_alive, ended, log) = {
            let sim = context.battle().unwrap();
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
                _ => None, // Draw or both have units
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
    pub fn assert_winner(&self, expected: TeamSide) {
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
    }

    pub fn assert_draw(&self) {
        assert_eq!(
            self.winner, None,
            "Expected draw but {:?} won. Left: {}, Right: {}, Iterations: {}",
            self.winner, self.left_alive, self.right_alive, self.iterations
        );
    }

    pub fn assert_units_alive(&self, side: TeamSide, count: usize) {
        let actual = match side {
            TeamSide::Left => self.left_alive,
            TeamSide::Right => self.right_alive,
        };
        assert_eq!(
            actual, count,
            "Expected {} units alive on {:?} side but got {}",
            count, side, actual
        );
    }

    pub fn assert_action_count(&self, min: usize) {
        assert!(
            self.log.actions.len() >= min,
            "Expected at least {} actions but got {}",
            min,
            self.log.actions.len()
        );
    }

    pub fn assert_iterations(&self, count: usize) {
        assert_eq!(
            self.iterations, count,
            "Expected {} iterations but got {}",
            count, self.iterations
        );
    }
}

pub fn reaction_deal_damage(damage: i32) -> Reaction {
    Reaction {
        trigger: Trigger::BattleStart,
        actions: vec![
            Action::set_value(Box::new(Expression::i32(damage))),
            Action::deal_damage,
        ],
    }
}
