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
        tape: &mut Option<Tape>,
    ) -> bool {
        Self::add_intro(resources, tape);
        let mut cluster = match tape {
            Some(_) => Some(NodeCluster::default()),
            None => None,
        };
        SlotSystem::move_to_slots_animated(world, resources, &mut cluster);
        Event::BattleStart.send(world, resources);
        Self::spin(world, resources, &mut cluster);
        if let Some(tape) = tape {
            tape.push(cluster.unwrap());
        }
        let mut ticks = 0;
        while Self::tick(world, resources, tape) && ticks < 1000 {
            ticks += 1;
        }
        Self::battle_won(world)
    }

    pub fn add_intro(resources: &Resources, tape: &mut Option<Tape>) {
        if let Some(tape) = tape.as_mut() {
            let mut node = Node::default();
            node.add_effects_by_key(
                TEAM_NAMES_KEY.to_string(),
                VfxSystem::vfx_battle_team_names_animation(resources),
            );
            let mut cluster = NodeCluster::new(node.finish_empty());
            cluster.set_duration(1.0);
            tape.push(cluster);
            tape.persistent_node.add_effects_by_key(
                TEAM_NAMES_KEY.to_string(),
                VfxSystem::vfx_battle_team_names(resources),
            );
        }
    }

    pub fn init_battle(
        light: &Team,
        dark: &Team,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::clear_world(world, resources);
        light.unpack(&Faction::Light, world, resources);
        dark.unpack(&Faction::Dark, world, resources);
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

    fn strickers_death_check(
        left: legion::Entity,
        right: legion::Entity,
        world: &legion::World,
    ) -> bool {
        UnitSystem::get_corpse(left, world).is_some()
            || UnitSystem::get_corpse(right, world).is_some()
    }

    pub fn tick(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) -> bool {
        let factions = &hashset! {Faction::Light, Faction::Dark};
        SlotSystem::fill_gaps(world, resources, factions);
        let mut cluster = match tape {
            Some(_) => Some(NodeCluster::default()),
            None => None,
        };
        Self::spin(world, resources, &mut cluster);
        SlotSystem::move_to_slots_animated(world, resources, &mut cluster);
        if let Some((left, right)) = Self::find_hitters(world) {
            Event::TurnStart.send(world, resources);
            Self::spin(world, resources, &mut cluster);
            if Self::strickers_death_check(left, right, world) {
                return true;
            }

            Event::BeforeStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            Self::spin(world, resources, &mut cluster);
            if Self::strickers_death_check(left, right, world) {
                return true;
            }

            Event::BeforeStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            Self::spin(world, resources, &mut cluster);
            if Self::strickers_death_check(left, right, world) {
                return true;
            }
            if let Some(tape) = tape {
                tape.push({
                    let mut cluster = cluster.unwrap();
                    cluster.set_duration(0.5);
                    cluster
                });
                cluster = Some(NodeCluster::default());
            }

            let (left_hit_pos, right_hit_pos) = (vec2(-1.0, 0.0), vec2(1.0, 0.0));

            if let Some(tape) = tape {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    left_hit_pos,
                    &mut node,
                    world,
                    EasingType::Linear,
                    0.03,
                );
                VfxSystem::translate_animated(
                    right,
                    right_hit_pos,
                    &mut node,
                    world,
                    EasingType::Linear,
                    0.03,
                );
                let mut cluster = NodeCluster::new(node.finish_full(world, resources));
                cluster.set_duration(0.5);
                tape.push(cluster);
            }

            let duration = 1.5;
            if let Some(cluster) = &mut cluster {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    SlotSystem::get_position(1, &Faction::Light),
                    &mut node,
                    world,
                    EasingType::QuartOut,
                    duration,
                );
                VfxSystem::translate_animated(
                    right,
                    SlotSystem::get_position(1, &Faction::Dark),
                    &mut node,
                    world,
                    EasingType::QuartOut,
                    duration,
                );
                Self::add_strike_vfx(world, resources, &mut node);
                cluster.push(node.finish_full(world, resources));
            }
            Self::hit(left, right, &mut cluster, world, resources);
            Self::spin(world, resources, &mut cluster);
            Event::TurnEnd.send(world, resources);
            Self::spin(world, resources, &mut cluster);
            if let Some(tape) = tape {
                let mut cluster = cluster.unwrap();
                cluster.set_duration(duration);
                tape.push(cluster);
            }
            return true;
        }
        false
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

    pub fn spin(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        let factions = hashset! {Faction::Light, Faction::Dark};
        ActionSystem::run_ticks(world, resources, cluster);
        Self::death_check(&factions, world, resources, cluster);
    }

    pub fn hit(
        left: legion::Entity,
        right: legion::Entity,
        cluster: &mut Option<NodeCluster>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::spin(world, resources, cluster);
        if let Ok(mut context_left) = ContextSystem::try_get_context(left, world) {
            if let Ok(mut context_right) = ContextSystem::try_get_context(right, world) {
                context_left.owner = left;
                context_left.target = right;
                resources.action_queue.push_back(Action::new(
                    context_left,
                    Effect::Damage {
                        value: None,
                        on_hit: None,
                    }
                    .wrap(),
                ));
                context_right.owner = right;
                context_right.target = left;
                resources.action_queue.push_back(Action::new(
                    context_right,
                    Effect::Damage {
                        value: None,
                        on_hit: None,
                    }
                    .wrap(),
                ));
            }
        }

        Self::spin(world, resources, cluster);

        if UnitSystem::get_corpse(left, world).is_none() {
            Event::AfterStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            Self::spin(world, resources, cluster);
        }

        if UnitSystem::get_corpse(right, world).is_none() {
            Event::AfterStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            Self::spin(world, resources, cluster);
        }
    }

    pub fn death_check(
        factions: &HashSet<Faction>,
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        ContextSystem::refresh_factions(factions, world, resources);
        let mut corpses = Vec::default();
        while let Some(dead_unit) = <(&EntityComponent, &Context)>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
            .filter_map(|(entity, context)| {
                match context.vars.get_int(&VarName::HpValue)
                    <= context.vars.get_int(&VarName::HpDamage)
                {
                    true => Some(entity.entity),
                    false => None,
                }
            })
            .choose(&mut thread_rng())
        {
            resources
                .logger
                .log(&format!("{:?} dead", dead_unit), &LogContext::UnitCreation);
            if UnitSystem::process_death(dead_unit, world, resources, cluster) {
                resources.logger.log(
                    &format!("{:?} removed", dead_unit),
                    &LogContext::UnitCreation,
                );
                corpses.push(dead_unit);
            }
        }
        for entity in corpses {
            Event::UnitDeath { target: entity }.send(world, resources);
        }
    }

    fn add_strike_vfx(world: &mut legion::World, resources: &mut Resources, node: &mut Node) {
        let position = BATTLEFIELD_POSITION;
        node.add_effect(VfxSystem::vfx_strike(resources, position));
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
                format!("Floor #{}", resources.floors.current_ind()),
                resources.fonts.get_font(1),
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
