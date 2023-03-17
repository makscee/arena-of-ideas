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
        let mut ticks = 0;
        while Self::tick(world, resources) && ticks < 1000 {
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
        Self::clear_world(world, resources);
        Self::create_enemies(resources, world);
        Self::create_team(resources, world);
        SlotSystem::fill_gaps(world, resources, hashset! {Faction::Dark, Faction::Light});
    }

    pub fn battle_won(world: &legion::World) -> bool {
        <&UnitComponent>::query()
            .iter(world)
            .filter(|unit| unit.faction == Faction::Dark)
            .count()
            == 0
    }

    pub fn finish_battle(world: &mut legion::World, resources: &mut Resources) {
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

    pub fn create_enemies(resources: &mut Resources, world: &mut legion::World) {
        Floors::load(world, resources);
    }

    pub fn create_team(resources: &mut Resources, world: &mut legion::World) {
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
        SlotSystem::fill_gaps(world, resources, hashset! {Faction::Light, Faction::Dark});
        Self::refresh_cassette(world, resources);
        Self::move_to_slots_animated(world, resources);
        Self::refresh_cassette(world, resources);
        resources.cassette.close_node();
        ActionSystem::run_ticks(world, resources);

        if let Some((left_entity, right_entity)) = Self::find_hitters(world) {
            Self::refresh_cassette(world, resources);
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

            Self::hit(left_entity, right_entity, world, resources);

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

            Self::clear_dead(world, resources);
            Self::refresh_cassette(world, resources);
            SlotSystem::refresh_slots_uniforms(world, &resources.options);
            return true;
        }
        return false;
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
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let context_left = Context {
            owner: left,
            target: right,
            parent: Some(left),
            ..ContextSystem::get_context(left, world)
        };
        resources.action_queue.push_back(Action::new(
            context_left.clone(),
            Effect::Damage { value: None },
        ));

        let context_right = Context {
            owner: right,
            target: left,
            parent: Some(right),
            ..ContextSystem::get_context(right, world)
        };
        resources.action_queue.push_back(Action::new(
            context_right.clone(),
            Effect::Damage { value: None },
        ));
        ActionSystem::run_ticks(world, resources);
        Event::AfterStrike {
            context: context_left,
        }
        .send(resources, world);
        Event::AfterStrike {
            context: context_right,
        }
        .send(resources, world);
        ActionSystem::run_ticks(world, resources);
    }

    pub fn clear_dead(world: &mut legion::World, resources: &mut Resources) {
        while let Some(dead_unit) = <(&EntityComponent, &HpComponent)>::query()
            .iter(world)
            .filter_map(|(unit, hp)| match hp.current <= 0 {
                true => Some(unit.entity),
                false => None,
            })
            .choose(&mut thread_rng())
        {
            resources.logger.log(
                &format!("Entity#{:?} dead", dead_unit),
                &LogContext::UnitCreation,
            );
            let context = ContextSystem::get_context(dead_unit, world);
            Event::BeforeDeath { context }.send(resources, world);
            ActionSystem::run_ticks(world, resources);
            if world
                .entry(dead_unit)
                .unwrap()
                .get_component::<HpComponent>()
                .unwrap()
                .current
                <= 0
            {
                UnitSystem::kill(dead_unit, world, resources);
            }
        }
    }

    fn refresh_cassette(world: &mut legion::World, resources: &mut Resources) {
        ContextSystem::refresh_all(world, resources);
        let factions = hashset! {Faction::Light, Faction::Dark};
        UnitSystem::draw_all_units_to_cassette_node(
            world,
            &resources.options,
            &resources.status_pool,
            &mut resources.cassette.node_template,
            factions,
        );
        resources
            .cassette
            .node_template
            .add_effects_by_key("slots", SlotSystem::get_slot_filled_visual_effects(world));
        resources.cassette.merge_template_into_last();
    }

    fn move_to_slots_animated(world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &mut AreaComponent, &EntityComponent)>::query()
            .iter_mut(world)
            .for_each(|(unit, area, entity)| {
                let slot_position = SlotSystem::get_unit_position(unit);
                if slot_position != area.position {
                    resources.cassette.add_effect(VfxSystem::vfx_move_unit(
                        entity.entity,
                        area.position,
                        slot_position,
                    ));
                    area.position = slot_position;
                }
            });
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
            .get_component::<AreaComponent>()
            .unwrap()
            .position;
        let right_position = world
            .entry_ref(right)
            .expect("Right striker not found")
            .get_component::<AreaComponent>()
            .unwrap()
            .position;
        let position = left_position + (right_position - left_position) * 0.5;
        resources
            .cassette
            .add_effect(VfxSystem::vfx_strike(resources, position));
    }

    fn add_strike_animation(
        cassette: &mut Cassette,
        phase: StrikePhase,
        entity: legion::Entity,
        faction: Faction,
    ) {
        match phase {
            StrikePhase::Charge => {
                cassette.add_effect(VfxSystem::vfx_strike_charge(entity, &faction))
            }
            StrikePhase::Release => {
                cassette.add_effect(VfxSystem::vfx_strike_release(entity, &faction))
            }
            StrikePhase::Retract => {
                cassette.add_effect(VfxSystem::vfx_strike_retract(entity, &faction))
            }
        };
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
