use crate::resources::Widget;

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
    ) -> usize {
        // Ladder::track_team(world, resources);
        Self::create_tape_entities(world, resources, tape);
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
        while Self::tick(world, resources, tape) && ticks < 100 {
            Self::update_score(world, resources, tape);
            ticks += 1;
        }
        if ticks == 1000 {
            panic!("Exceeded ticks limit");
        }
        Self::clear_tape_entities(world, resources, tape);
        let score = Ladder::get_score(world, resources);
        Self::add_outro(score, world, resources, tape);
        score
    }

    fn create_tape_entities(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) {
        if tape.is_none() {
            return;
        }

        // let entity = Self::push_tape_shader_entity(
        //     resources.options.shaders.battle_score_indicator.clone(),
        //     world,
        // );
        // resources.battle_data.score_entity = Some(entity);
        // Self::update_score(world, resources, tape);

        let (left, right) = VfxSystem::vfx_battle_team_names(world, resources);
        let names = (
            Self::push_tape_shader_entity(left, world),
            Self::push_tape_shader_entity(right, world),
        );
        resources.battle_data.team_names_entitities = Some(names);
    }

    fn push_tape_shader_entity(shader: Shader, world: &mut legion::World) -> legion::Entity {
        let entity = world.push((shader, TapeEntityComponent {}));
        world
            .entry(entity)
            .unwrap()
            .add_component(EntityComponent::new(entity));
        entity
    }

    fn clear_tape_entities(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) {
        if tape.is_none() {
            return;
        }
        if let Some(entity) = resources.battle_data.score_entity {
            world.remove(entity);
        }
        if let Some((left, right)) = resources.battle_data.team_names_entitities {
            world.remove(left);
            world.remove(right);
        }
    }

    fn update_score(world: &mut legion::World, resources: &Resources, tape: &mut Option<Tape>) {
        return;
        if tape.is_none() {
            return;
        }
        // let score = Ladder::get_score_units(world, resources);
        let score = (0, 1);
        if let Some(mut entry) = resources
            .battle_data
            .score_entity
            .and_then(|x| world.entry(x))
        {
            if let Ok(shader) = entry.get_component_mut::<Shader>() {
                shader.set_uniform_ref(
                    "u_text",
                    ShaderUniform::String((1, format!("{}/{}", score.0, score.1))),
                );
            }
        }
    }

    fn add_outro(
        score: usize,
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) {
        if let Some(tape) = tape {
            let node = Widget::BattleOverPanel {
                score,
                options: &resources.options,
            }
            .generate_node();
            tape.push(NodeCluster::new(
                node.lock(NodeLockType::Full { world, resources }),
            ));
        }
    }

    pub fn init_battle(
        light: &PackedTeam,
        dark: &PackedTeam,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::clear_world(world, resources);
        light.unpack(&Faction::Light, world, resources);
        dark.unpack(&Faction::Dark, world, resources);
    }

    pub fn finish_floor_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.last_score = Ladder::get_score(world, resources);
        resources.last_round = resources.ladder.current_ind();
        resources.total_score += resources.last_score;
        if resources.last_score > 0 {
            if resources.ladder.next() {
                resources.transition_state = GameState::Shop;
            } else {
                resources.transition_state = GameState::GameOver;
            }
        } else {
            resources.transition_state = GameState::GameOver;
        }
        Self::clear_world(world, resources);
    }

    pub fn clear_world(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(world, &Faction::battle());
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
        for faction in Faction::battle() {
            SlotSystem::fill_gaps(faction, world);
        }
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
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }

            Event::BeforeStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            Self::spin(world, resources, &mut cluster);
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }

            Event::BeforeStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            Self::spin(world, resources, &mut cluster);
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
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

            let scale = resources.options.floats.slots_striker_scale;
            let (left_hit_pos, right_hit_pos) = (vec2(-1.0 * scale, 0.0), vec2(1.0 * scale, 0.0));

            if let Some(tape) = tape {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    left_hit_pos,
                    &mut node,
                    world,
                    resources,
                    EasingType::Linear,
                    0.03,
                );
                VfxSystem::translate_animated(
                    right,
                    right_hit_pos,
                    &mut node,
                    world,
                    resources,
                    EasingType::Linear,
                    0.03,
                );
                let mut cluster =
                    NodeCluster::new(node.lock(NodeLockType::Full { world, resources }));
                cluster.push(Node::default().lock(NodeLockType::Full { world, resources }));
                cluster.set_duration(0.5);
                tape.push(cluster);
            }

            let duration = 1.5;
            if let Some(cluster) = &mut cluster {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    SlotSystem::get_position(1, &Faction::Light, resources),
                    &mut node,
                    world,
                    resources,
                    EasingType::QuartOut,
                    duration,
                );
                VfxSystem::translate_animated(
                    right,
                    SlotSystem::get_position(1, &Faction::Dark, resources),
                    &mut node,
                    world,
                    resources,
                    EasingType::QuartOut,
                    duration,
                );
                Self::add_strike_vfx(world, resources, &mut node);
                cluster.push(node.lock(NodeLockType::Full { world, resources }));
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
        let mut light = None;
        let mut dark = None;
        for (entity, state) in <(&EntityComponent, &ContextState)>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
        {
            if state.get_int(&VarName::Slot, world) == 1 {
                if state.get_faction(&VarName::Faction, world) == Faction::Light {
                    light = Some(entity.entity)
                } else if state.get_faction(&VarName::Faction, world) == Faction::Dark {
                    dark = Some(entity.entity)
                }
            }
        }
        if light.is_some() && dark.is_some() {
            Some((light.unwrap(), dark.unwrap()))
        } else {
            None
        }
    }

    pub fn spin(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        ActionSystem::run_ticks(world, resources, cluster);
        Self::death_check(world, resources, cluster);
    }

    pub fn hit(
        left: legion::Entity,
        right: legion::Entity,
        cluster: &mut Option<NodeCluster>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::spin(world, resources, cluster);

        if UnitSystem::is_alive(left, world, resources)
            && UnitSystem::is_alive(right, world, resources)
        {
            resources.action_queue.push_back(Action::new(
                Context::new(ContextLayer::Unit { entity: left }, world, resources)
                    .set_target(right),
                Effect::Damage {
                    value: None,
                    on_hit: None,
                }
                .wrap(),
            ));
            resources.action_queue.push_back(Action::new(
                Context::new(ContextLayer::Unit { entity: right }, world, resources)
                    .set_target(left),
                Effect::Damage {
                    value: None,
                    on_hit: None,
                }
                .wrap(),
            ));
            Self::spin(world, resources, cluster);
        }

        if UnitSystem::is_alive(left, world, resources) {
            Event::AfterStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            Self::spin(world, resources, cluster);
        }

        if UnitSystem::is_alive(right, world, resources) {
            Event::AfterStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            Self::spin(world, resources, cluster);
        }
    }

    pub fn death_check(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        let mut corpses = Vec::default();
        while let Some(dead_unit) = <&EntityComponent>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
            .filter_map(|entity| {
                let context = Context::new(
                    ContextLayer::Unit {
                        entity: entity.entity,
                    },
                    world,
                    resources,
                );
                match context.get_int(&VarName::HpValue, world)
                    <= context.get_int(&VarName::HpDamage, world)
                {
                    true => Some(entity.entity),
                    false => None,
                }
            })
            .choose(&mut thread_rng())
        {
            resources.logger.log(
                || format!("{:?} dead", dead_unit),
                &LogContext::UnitCreation,
            );
            if UnitSystem::process_death(dead_unit, world, resources, cluster) {
                resources.logger.log(
                    || format!("{:?} removed", dead_unit),
                    &LogContext::UnitCreation,
                );
                corpses.push(dead_unit);
            }
        }
        for entity in corpses {
            Event::UnitDeath { target: entity }.send(world, resources);
        }
    }

    fn add_strike_vfx(_: &mut legion::World, resources: &mut Resources, node: &mut Node) {
        let position = BATTLEFIELD_POSITION;
        node.add_effect(VfxSystem::vfx_strike(resources, position));
    }
}

impl System for BattleSystem {
    fn ui<'a>(
        &'a mut self,
        _: &'a ui::Controller,
        _: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        Box::new(
            (Text::new(
                format!("Level #{}", resources.ladder.current_ind()),
                resources.fonts.get_font(1),
                70.0,
                Rgba::WHITE,
            ),)
                .column()
                .flex_align(vec2(Some(1.0), None), vec2(1.0, 1.0))
                .uniform_padding(32.0),
        )
    }
}
