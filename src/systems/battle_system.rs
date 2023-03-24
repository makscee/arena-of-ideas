use super::*;
use geng::ui::*;

pub struct BattleSystem {}

impl BattleSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_battle(
        world: &mut legion::World,
        resources: &mut Resources,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) -> bool {
        let mut ticks = 0;
        Event::BattleStart.send(world, resources);
        while Self::tick(world, resources, nodes) && ticks < 1000 {
            ticks += 1;
        }
        Self::battle_won(world)
    }

    pub fn init_battle(world: &mut legion::World, resources: &mut Resources) {
        Self::clear_world(world, resources);
        TeamPool::unpack_team(&Faction::Light, world, resources);
        TeamPool::unpack_team(&Faction::Dark, world, resources);
    }

    pub fn battle_won(world: &legion::World) -> bool {
        <&UnitComponent>::query()
            .iter(world)
            .filter(|unit| unit.faction == Faction::Dark)
            .count()
            == 0
    }

    pub fn finish_floor_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.game_won = Self::battle_won(world);
        resources.last_round = resources.floors.current_ind();
        if !resources.game_won {
            resources.transition_state = GameState::GameOver;
        } else {
            if resources.floors.next() {
                resources.transition_state = GameState::Shop;
            } else {
                resources.transition_state = GameState::GameOver;
            }
        }
        Self::clear_world(world, resources);
    }

    pub fn clear_world(world: &mut legion::World, resources: &mut Resources) {
        let factions = &hashset! {Faction::Dark, Faction::Light};
        UnitSystem::clear_factions(world, resources, factions);
    }

    pub fn save_floor(resources: &mut Resources) {
        let team = resources.floors.current().clone();
        let faction = Faction::Dark;
        TeamPool::save_team(faction, team, resources);
    }

    pub fn save_player_team(resources: &mut Resources) {
        let team = TeamPool::get_team(Faction::Team, resources).clone();
        let faction = Faction::Light;
        TeamPool::save_team(faction, team, resources);
    }

    pub fn tick(
        world: &mut legion::World,
        resources: &mut Resources,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) -> bool {
        let factions = &hashset! {Faction::Light, Faction::Dark};
        SlotSystem::fill_gaps(world, resources, factions);
        ContextSystem::refresh_factions(factions, world, resources);
        ActionSystem::run_ticks(world, resources, nodes);
        SlotSystem::fill_gaps(world, resources, factions);
        SlotSystem::move_to_slots_animated(world, resources, nodes);
        if let Some((left, right)) = Self::find_hitters(world) {
            Event::TurnStart.send(world, resources);
            ActionSystem::run_ticks(world, resources, nodes);
            Self::move_strikers(&StrikePhase::Charge, left, right, world, resources, nodes);
            Self::move_strikers(&StrikePhase::Release, left, right, world, resources, nodes);
            Self::add_strike_vfx(world, resources, nodes);
            Self::hit(left, right, nodes, world, resources);
            Self::death_check(world, resources, nodes);
            Self::move_strikers(&StrikePhase::Retract, left, right, world, resources, nodes);
            Event::TurnEnd.send(world, resources);
            ActionSystem::run_ticks(world, resources, nodes);
            Self::death_check(world, resources, nodes);
            return true;
        }
        false
    }

    fn move_strikers(
        phase: &StrikePhase,
        left: legion::Entity,
        right: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) {
        if let Some(nodes) = nodes {
            let mut node = CassetteNode::default();
            let (left_pos, right_pos) = Self::get_strikers_positions(phase);
            let (easing, duration) = match phase {
                StrikePhase::Charge => (EasingType::QuartInOut, 1.5),
                StrikePhase::Release => (EasingType::Linear, 0.1),
                StrikePhase::Retract => (EasingType::QuartOut, 0.25),
            };
            VfxSystem::translate_animated(left, left_pos, &mut node, world, easing, duration);
            VfxSystem::translate_animated(right, right_pos, &mut node, world, easing, duration);
            nodes.push(node.finish(world, resources));
        }
    }

    fn get_strikers_positions(phase: &StrikePhase) -> (vec2<f32>, vec2<f32>) {
        let left = vec2(-1.0, 1.0);
        let right = vec2(1.0, 1.0);
        let left_slot = SlotSystem::get_position(1, &Faction::Light);
        let right_slot = SlotSystem::get_position(1, &Faction::Dark);

        let delta = match phase {
            StrikePhase::Charge => vec2(4.5, 0.0),
            StrikePhase::Release => vec2(-right_slot.x + 1.0, 0.0),
            StrikePhase::Retract => vec2::ZERO,
        };
        (delta * left + left_slot, delta * right + right_slot)
    }

    pub fn find_hitters(world: &legion::World) -> Option<(legion::Entity, legion::Entity)> {
        let units = <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .collect_vec();

        units
            .iter()
            .find(|(unit, _)| unit.slot == 1 && unit.faction == Faction::Light)
            .and_then(|(_, left)| {
                match units
                    .iter()
                    .find(|(unit, _)| unit.slot == 1 && unit.faction == Faction::Dark)
                {
                    Some((_, right)) => Some((left.entity, right.entity)),
                    None => None,
                }
            })
    }

    pub fn hit(
        left: legion::Entity,
        right: legion::Entity,
        nodes: &mut Option<Vec<CassetteNode>>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let context_left = Context {
            owner: left,
            target: right,
            ..ContextSystem::get_context(left, world)
        };
        resources.action_queue.push_back(Action::new(
            context_left.clone(),
            Effect::Damage {
                value: None,
                on_hit: None,
            }
            .wrap(),
        ));
        ActionSystem::run_ticks(world, resources, nodes);
        Event::AfterStrike {
            owner: left,
            target: right,
        }
        .send(world, resources);
        ActionSystem::run_ticks(world, resources, nodes);
        let context_right = Context {
            owner: right,
            target: left,
            ..ContextSystem::get_context(right, world)
        };
        resources.action_queue.push_back(Action::new(
            context_right.clone(),
            Effect::Damage {
                value: None,
                on_hit: None,
            }
            .wrap(),
        ));
        ActionSystem::run_ticks(world, resources, nodes);
        Event::AfterStrike {
            owner: right,
            target: left,
        }
        .send(world, resources);
        ActionSystem::run_ticks(world, resources, nodes);
    }

    pub fn death_check(
        world: &mut legion::World,
        resources: &mut Resources,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) {
        ContextSystem::refresh_all(world, resources);
        while let Some(dead_unit) = <(&EntityComponent, &Context, &HealthComponent)>::query()
            .iter(world)
            .filter_map(|(unit, context, _)| {
                match context.vars.get_int(&VarName::HpValue)
                    <= context.vars.get_int(&VarName::HpDamage)
                {
                    true => Some(unit.entity),
                    false => None,
                }
            })
            .choose(&mut thread_rng())
        {
            resources
                .logger
                .log(&format!("{:?} dead", dead_unit), &LogContext::UnitCreation);
            if UnitSystem::process_death(dead_unit, world, resources, nodes) {
                resources.logger.log(
                    &format!("{:?} removed", dead_unit),
                    &LogContext::UnitCreation,
                );
            }
        }
    }

    fn add_strike_vfx(
        world: &mut legion::World,
        resources: &mut Resources,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) {
        if let Some(nodes) = nodes.as_mut() {
            let position = BATTLEFIELD_POSITION;
            let mut node: CassetteNode = default();
            node.add_effect(VfxSystem::vfx_strike(resources, position));
            nodes.push(node.finish(world, resources));
        }
    }
}

impl System for BattleSystem {
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        Box::new(
            (Text::new(
                format!("Round #{}", resources.floors.current_ind()),
                resources.fonts.get_font(0),
                70.0,
                Rgba::WHITE,
            ),)
                .column()
                .flex_align(vec2(Some(1.0), None), vec2(1.0, 1.0))
                .uniform_padding(32.0),
        )
    }

    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {}
}

enum StrikePhase {
    Charge,
    Release,
    Retract,
}
