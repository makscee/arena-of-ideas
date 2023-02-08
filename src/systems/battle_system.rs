use legion::EntityStore;

use super::*;

pub struct BattleSystem {}

impl BattleSystem {
    pub fn run_battle(world: &mut legion::World, resources: &mut Resources) {
        Self::init_battle(world, resources);
        let mut ticks = 0;
        while Self::tick(world, resources) && ticks < 100 {
            ticks += 1;
        }
        resources.cassette.node_template.clear();
        UnitComponent::add_all_units_to_node_template(world, resources);
        Self::finish_battle(world);
    }

    pub fn init_battle(world: &mut legion::World, resources: &mut Resources) {
        Self::init_units(resources, world);
        Self::init_statuses(resources);
        while ActionSystem::tick(world, resources) {}
    }

    pub fn finish_battle(world: &mut legion::World) {
        let unit_entitites = <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .map(|(entity, _)| entity.entity.clone())
            .collect_vec();
        unit_entitites.iter().for_each(|entity| {
            world.remove(*entity);
        });
    }

    fn init_units(resources: &mut Resources, world: &mut legion::World) {
        let left = resources.unit_templates.values().collect_vec()[0].create_unit_entity(
            world,
            &mut resources.statuses,
            Faction::Light,
        );
        let mut left = world.entry(left).unwrap();
        left.get_component_mut::<Position>().unwrap().0 = vec2(-1.5, 0.0);

        let right = resources.unit_templates.values().collect_vec()[1].create_unit_entity(
            world,
            &mut resources.statuses,
            Faction::Dark,
        );
        let mut right = world.entry(right).unwrap();
        right.get_component_mut::<Position>().unwrap().0 = vec2(1.5, 0.0);
    }

    fn init_statuses(resources: &mut Resources) {
        let statuses = resources
            .statuses
            .active_statuses
            .iter()
            .map(|(entity, map)| (entity.clone(), map.clone()))
            .collect_vec();
        statuses.iter().for_each(|(_entity, statuses)| {
            statuses.iter().for_each(|(status, context)| {
                Event::Init {
                    status: status.to_string(),
                }
                .send(&context.clone(), resources)
                .expect("Error on status Init");
            })
        })
    }

    pub fn tick(world: &mut legion::World, resources: &mut Resources) -> bool {
        let units = <(&UnitComponent, &EntityComponent, &Faction)>::query()
            .iter(world)
            .collect_vec();
        let left = units
            .iter()
            .find(|(_, _, faction)| **faction == Faction::Light);
        let right = units
            .iter()
            .find(|(_, _, faction)| **faction == Faction::Dark);
        if left.is_some() && right.is_some() && units.len() > 1 {
            let left_entity = left.unwrap().1.entity;
            let right_entity = right.unwrap().1.entity;

            resources.cassette.node_template.clear();
            UnitComponent::add_all_units_to_node_template(world, resources);
            resources.cassette.close_node();

            Self::add_strike_animation(
                &mut resources.cassette,
                StrikePhase::Charge,
                left_entity,
                Faction::Light,
            );
            Self::add_strike_animation(
                &mut resources.cassette,
                StrikePhase::Charge,
                right_entity,
                Faction::Dark,
            );
            resources.cassette.close_node();

            Self::add_strike_animation(
                &mut resources.cassette,
                StrikePhase::Release,
                left_entity,
                Faction::Light,
            );
            Self::add_strike_animation(
                &mut resources.cassette,
                StrikePhase::Release,
                right_entity,
                Faction::Dark,
            );
            resources.cassette.node_template.clear();
            resources
                .cassette
                .node_template
                .add_effect(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityShaderConst {
                        entity: left_entity,
                        uniforms: hashmap! {
                            "u_position" => ShaderUniform::Vec2(vec2(-1.0,0.0))
                        }
                        .into(),
                    },
                    -15,
                ));
            resources
                .cassette
                .node_template
                .add_effect(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityShaderConst {
                        entity: right_entity,
                        uniforms: hashmap! {
                            "u_position" => ShaderUniform::Vec2(vec2(1.0,0.0))
                        }
                        .into(),
                    },
                    -15,
                ));
            resources.cassette.close_node();

            let context = Context {
                owner: left_entity,
                target: right_entity,
                creator: left_entity,
                vars: default(),
                status: default(),
            };
            resources
                .action_queue
                .push_back(Action::new(context, Effect::Damage { value: None }));
            let context = Context {
                owner: right_entity,
                target: left_entity,
                creator: right_entity,
                vars: default(),
                status: default(),
            };
            resources
                .action_queue
                .push_back(Action::new(context, Effect::Damage { value: None }));
            let mut ticks = 0;
            while ActionSystem::tick(world, resources) && ticks < 1000 {
                ticks += 1;
            }

            UnitComponent::add_all_units_to_node_template(world, resources); // get changes (like stats) after ActionSystem execution
            resources.cassette.merge_template_into_last(); // merge this changes into last node, to display changed HP alongside any extra effects from ActionSystem
            Self::add_strike_vfx(world, resources, left_entity, right_entity);
            resources.cassette.close_node();

            Self::add_strike_animation(
                &mut resources.cassette,
                StrikePhase::Retract,
                left_entity,
                Faction::Light,
            );
            Self::add_strike_animation(
                &mut resources.cassette,
                StrikePhase::Retract,
                right_entity,
                Faction::Dark,
            );
            resources.cassette.close_node();

            let dead_units = <(&EntityComponent, &HpComponent)>::query()
                .iter(world)
                .filter_map(|(unit, hp)| match hp.current() <= 0 {
                    true => Some(unit.entity),
                    false => None,
                })
                .collect_vec();
            if !dead_units.is_empty() {
                dead_units.iter().for_each(|entity| {
                    debug!("Entity#{:?} dead", entity);
                    world.remove(*entity);
                });
            }
            return true;
        }
        return false;
    }

    fn add_strike_vfx(
        world: &legion::World,
        resources: &mut Resources,
        left: legion::Entity,
        right: legion::Entity,
    ) {
        let left_position = world
            .entry_ref(left)
            .expect("Left striker not found")
            .get_component::<Position>()
            .unwrap()
            .0;
        let right_position = world
            .entry_ref(right)
            .expect("Right striker not found")
            .get_component::<Position>()
            .unwrap()
            .0;
        let position = left_position + (right_position - left_position) * 0.5;
        resources.cassette.add_effect(VisualEffect {
            duration: 0.5,
            r#type: VisualEffectType::ShaderAnimation {
                shader: resources.options.strike.clone(),
                from: hashmap! {
                    "u_time" => ShaderUniform::Float(0.0),
                    "u_position" => ShaderUniform::Vec2(position),
                }
                .into(),
                to: hashmap! {
                    "u_time" => ShaderUniform::Float(1.0),
                    "u_position" => ShaderUniform::Vec2(position),
                }
                .into(),
                easing: EasingType::Linear,
            },
            order: 0,
        })
    }

    fn add_strike_animation(
        cassette: &mut Cassette,
        phase: StrikePhase,
        entity: legion::Entity,
        faction: Faction,
    ) {
        let faction_mul = match faction {
            Faction::Light => -1.0,
            Faction::Dark => 1.0,
        };
        match phase {
            StrikePhase::Charge => cassette.add_effect(VisualEffect::new(
                1.5,
                VisualEffectType::EntityShaderAnimation {
                    entity,
                    from: hashmap! {
                        "u_position" => ShaderUniform::Vec2(vec2(1.5, 0.0) * faction_mul),
                    }
                    .into(),
                    to: hashmap! {
                        "u_position" => ShaderUniform::Vec2(vec2(4.5, 0.0) * faction_mul),
                    }
                    .into(),
                    easing: EasingType::QuartInOut,
                },
                -10,
            )),
            StrikePhase::Release => cassette.add_effect(VisualEffect::new(
                0.1,
                VisualEffectType::EntityShaderAnimation {
                    entity,
                    from: hashmap! {
                        "u_position" => ShaderUniform::Vec2(vec2(4.5, 0.0) * faction_mul),
                    }
                    .into(),
                    to: hashmap! {
                        "u_position" => ShaderUniform::Vec2(vec2(1.0, 0.0) * faction_mul),
                    }
                    .into(),
                    easing: EasingType::Linear,
                },
                -10,
            )),
            StrikePhase::Retract => cassette.add_effect(VisualEffect::new(
                0.25,
                VisualEffectType::EntityShaderAnimation {
                    entity,
                    from: hashmap! {
                        "u_position" => ShaderUniform::Vec2(vec2(1.0, 0.0) * faction_mul),
                    }
                    .into(),
                    to: hashmap! {
                        "u_position" => ShaderUniform::Vec2(vec2(1.5, 0.0) * faction_mul),
                    }
                    .into(),
                    easing: EasingType::QuartOut,
                },
                -10,
            )),
        };
    }
}

enum StrikePhase {
    Charge,
    Release,
    Retract,
}
