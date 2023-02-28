use legion::EntityStore;

use super::*;
use geng::ui::*;

pub struct BattleSystem {}

const STRIKE_FREEZE_EFFECT_KEY: &str = "strike_freeze";
impl BattleSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_battle(world: &mut legion::World, resources: &mut Resources) {
        Self::init_battle(world, resources);
        let mut ticks = 0;
        while Self::tick(world, resources) && ticks < 100 {
            ticks += 1;
        }
        resources.cassette.node_template.clear();
        UnitSystem::draw_all_units_to_cassette_node(
            world,
            &resources.options,
            &resources.status_pool,
            &mut resources.cassette.node_template,
            hashset! {Faction::Dark, Faction::Light},
        );
    }

    pub fn init_battle(world: &mut legion::World, resources: &mut Resources) {
        WorldSystem::set_var(
            world,
            VarName::RoundNumber,
            &Var::Int(resources.rounds.next_round as i32 + 1),
        );
        SlotSystem::refresh_slot_shaders(world, resources, hashset![Faction::Dark, Faction::Light]);
        Self::create_enemies(resources, world);
        Self::create_team(resources, world);
        ContextSystem::refresh_all(world);
        ActionSystem::run_ticks(world, resources);
    }

    fn battle_won(world: &legion::World) -> bool {
        <&UnitComponent>::query()
            .iter(world)
            .filter(|unit| unit.faction == Faction::Dark)
            .count()
            == 0
    }

    pub fn finish_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.game_won = Self::battle_won(world);
        resources.last_round = resources.rounds.next_round;
        if !resources.game_won {
            resources.transition_state = GameState::GameOver;
        } else {
            if resources.rounds.next_round == ROUNDS_COUNT {
                resources.transition_state = GameState::GameOver;
            } else {
                resources.transition_state = GameState::Shop;
            }
        }
        let factions = &hashset! {Faction::Dark, Faction::Light};
        resources.status_pool.clear_factions(factions, world);
        WorldSystem::clear_factions(world, factions);
    }

    fn create_enemies(resources: &mut Resources, world: &mut legion::World) {
        Rounds::load(world, resources);
    }

    fn create_team(resources: &mut Resources, world: &mut legion::World) {
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .filter_map(|(unit, entity)| {
                if unit.faction == Faction::Team {
                    Some(entity.entity)
                } else {
                    None
                }
            })
            .collect_vec()
            .iter()
            .for_each(|entity| {
                UnitSystem::duplicate_unit(*entity, world, resources, Faction::Light)
            });
    }

    pub fn tick(world: &mut legion::World, resources: &mut Resources) -> bool {
        resources.cassette.node_template.clear();
        resources.cassette.close_node();

        SlotSystem::fill_gaps(world, resources, hashset! {Faction::Light, Faction::Dark});
        let units = <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .collect_vec();
        let left = units
            .iter()
            .find(|(unit, _)| unit.slot == 1 && unit.faction == Faction::Light);
        let right = units
            .iter()
            .find(|(unit, _)| unit.slot == 1 && unit.faction == Faction::Dark);
        if left.is_some() && right.is_some() {
            Self::refresh_cassette(world, resources);
            resources.cassette.close_node();
            let left_entity = left.unwrap().1.entity;
            let right_entity = right.unwrap().1.entity;

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
            resources.cassette.node_template.add_effect_by_key(
                STRIKE_FREEZE_EFFECT_KEY,
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityShaderConst {
                        entity: left_entity,
                        uniforms: hashmap! {
                            "u_position" => ShaderUniform::Vec2(vec2(-1.0,0.0))
                        }
                        .into(),
                    },
                    15,
                ),
            );
            resources.cassette.node_template.add_effect_by_key(
                STRIKE_FREEZE_EFFECT_KEY,
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityShaderConst {
                        entity: right_entity,
                        uniforms: hashmap! {
                            "u_position" => ShaderUniform::Vec2(vec2(1.0,0.0))
                        }
                        .into(),
                    },
                    15,
                ),
            );
            resources.cassette.close_node();

            let context = Context {
                owner: left_entity,
                target: right_entity,
                parent: Some(left_entity),
                ..ContextSystem::get_context(left_entity, world)
            };
            resources
                .action_queue
                .push_back(Action::new(context, Effect::Damage { value: None }));

            let context = Context {
                owner: right_entity,
                target: left_entity,
                parent: Some(right_entity),
                ..ContextSystem::get_context(right_entity, world)
            };
            resources
                .action_queue
                .push_back(Action::new(context, Effect::Damage { value: None }));

            ActionSystem::run_ticks(world, resources);

            Self::refresh_cassette(world, resources);
            Self::add_strike_vfx(world, resources, left_entity, right_entity);
            resources.cassette.close_node();
            resources
                .cassette
                .node_template
                .clear_key(STRIKE_FREEZE_EFFECT_KEY);

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

            while let Some(dead_unit) = <(&EntityComponent, &HpComponent)>::query()
                .iter(world)
                .filter_map(|(unit, hp)| match hp.current <= 0 {
                    true => Some(unit.entity),
                    false => None,
                })
                .choose(&mut thread_rng())
            {
                debug!("Entity#{:?} dead", dead_unit);
                let context = ContextSystem::get_context(dead_unit, world);
                Event::BeforeDeath { context }.send(resources, world);
                ActionSystem::run_ticks(world, resources);
                Self::refresh_cassette(world, resources);
                if world
                    .entry(dead_unit)
                    .unwrap()
                    .get_component::<HpComponent>()
                    .unwrap()
                    .current
                    <= 0
                {
                    WorldSystem::kill(dead_unit, world, resources);
                }
            }
            return true;
        }
        return false;
    }

    fn refresh_cassette(world: &legion::World, resources: &mut Resources) {
        UnitSystem::draw_all_units_to_cassette_node(
            world,
            &resources.options,
            &resources.status_pool,
            &mut resources.cassette.node_template,
            hashset! {Faction::Light, Faction::Dark},
        );
        resources.cassette.merge_template_into_last();
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
            .get_component::<PositionComponent>()
            .unwrap()
            .0;
        let right_position = world
            .entry_ref(right)
            .expect("Right striker not found")
            .get_component::<PositionComponent>()
            .unwrap()
            .0;
        let position = left_position + (right_position - left_position) * 0.5;
        resources.cassette.add_effect(VisualEffect::new(
            0.5,
            VisualEffectType::ShaderAnimation {
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
            0,
        ));
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
            _ => panic!("Wrong faction"),
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
                20,
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
                20,
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
                20,
            )),
        };
    }
}

impl System for BattleSystem {
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        Box::new(
            (Text::new(
                format!("Round #{}", resources.rounds.next_round),
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
