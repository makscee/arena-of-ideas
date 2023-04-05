use super::*;

pub struct SlotSystem {}

pub const SLOTS_COUNT: usize = 5;
pub const PULL_FORCE: f32 = 7.0;
pub const SHOP_POSITION: vec2<f32> = vec2(-30.0, 0.0);
pub const SHOP_TEAM_OFFSET: vec2<f32> = vec2(0.0, -3.0);
pub const SHOP_CASE_OFFSET: vec2<f32> = vec2(0.0, 3.0);
pub const STRIKER_OFFSET: vec2<f32> = vec2(5.5, 0.0);
pub const TEAM_OFFSET: vec2<f32> = vec2(2.5, 4.0);
pub const BATTLEFIELD_POSITION: vec2<f32> = vec2(0.0, 0.0);

impl SlotSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_position(slot: usize, faction: &Faction) -> vec2<f32> {
        let faction_mul: vec2<f32> = vec2(
            match faction {
                Faction::Light => -1.0,
                Faction::Dark => 1.0,
                Faction::Team => -1.0,
                Faction::Shop => -1.0,
                Faction::Gallery => 0.0,
            },
            1.0,
        );
        match faction {
            Faction::Light | Faction::Dark => {
                return match slot == 1 {
                    true => STRIKER_OFFSET,
                    false => vec2(slot as f32 - 1.0, 1.0) * TEAM_OFFSET,
                } * faction_mul
                    + BATTLEFIELD_POSITION
            }
            Faction::Team => {
                SHOP_POSITION
                    + SHOP_TEAM_OFFSET
                    + vec2(slot as f32 - SLOTS_COUNT as f32 / 2.0, 0.0) * faction_mul * 2.5
            }
            Faction::Shop => {
                SHOP_POSITION
                    + SHOP_CASE_OFFSET
                    + vec2(slot as f32 - SLOTS_COUNT as f32 / 2.0, 0.0) * faction_mul * 2.5
            }
            Faction::Gallery => vec2::ZERO,
        }
    }

    pub fn get_unit_position(unit: &UnitComponent) -> vec2<f32> {
        Self::get_position(unit.slot, &unit.faction)
    }

    pub fn move_to_slots_animated(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        if let Some(cluster) = cluster {
            let mut node = Node::default();
            <(&EntityComponent, &UnitComponent, &AreaComponent)>::query()
                .iter(world)
                .filter_map(|(entity, unit, area)| {
                    let need_pos = Self::get_unit_position(unit);
                    match need_pos == area.position {
                        true => None,
                        false => Some((need_pos, entity.entity)),
                    }
                })
                .collect_vec()
                .into_iter()
                .for_each(|(need_pos, entity)| {
                    VfxSystem::translate_animated(
                        entity,
                        need_pos,
                        &mut node,
                        world,
                        EasingType::QuartInOut,
                        0.8,
                    )
                });
            cluster.push(node.lock(NodeLockType::Full { world, resources }));
        }
    }

    pub fn get_hovered_slot(faction: &Faction, mouse_pos: vec2<f32>) -> Option<usize> {
        for slot in 1..=SLOTS_COUNT {
            let slot_pos = Self::get_position(slot, faction);
            if (mouse_pos - slot_pos).len() < 1.5 {
                return Some(slot);
            }
        }
        None
    }

    pub fn get_slot_entity(world: &legion::World, faction: Faction, slot: usize) -> legion::Entity {
        <(&SlotComponent, &EntityComponent)>::query()
            .iter(world)
            .find_map(|(slot_component, entity)| {
                match slot_component.slot == slot && slot_component.faction == faction {
                    true => Some(entity.entity),
                    false => None,
                }
            })
            .unwrap()
    }

    pub fn find_unit_by_slot(
        slot: usize,
        faction: &Faction,
        world: &legion::World,
    ) -> Option<legion::Entity> {
        <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .find_map(
                |(entity, unit)| match unit.faction == *faction && unit.slot == slot {
                    true => Some(entity.entity),
                    false => None,
                },
            )
    }

    pub fn clear_world(world: &mut legion::World) {
        <(&SlotComponent, &EntityComponent)>::query()
            .iter(world)
            .map(|(_, entity)| entity.entity)
            .collect_vec()
            .iter()
            .for_each(|entity| {
                world.remove(*entity);
            });
    }

    pub fn init_world(world: &mut legion::World, options: &Options, factions: HashSet<Faction>) {
        Self::clear_world(world);
        for slot in 1..=SLOTS_COUNT {
            factions.iter().for_each(|faction| {
                let entity = world.push((
                    options
                        .shaders
                        .slot
                        .clone()
                        .set_uniform("u_color", ShaderUniform::Color(faction.color(&options)))
                        .set_uniform(
                            "u_position",
                            ShaderUniform::Vec2(Self::get_position(slot, faction)),
                        ),
                    SlotComponent {
                        slot,
                        faction: *faction,
                    },
                ));
                world
                    .entry(entity)
                    .unwrap()
                    .add_component(EntityComponent::new(entity));
            })
        }
        Self::refresh_slots_uniforms(world, options);
    }

    pub fn refresh_slots_uniforms(world: &mut legion::World, options: &Options) {
        let filled_slots = Self::get_filled_slots(world);
        <(&SlotComponent, &mut Shader)>::query()
            .iter_mut(world)
            .for_each(|(slot, shader)| {
                let filled = filled_slots.contains(&(slot.faction, slot.slot));
                shader.set_uniform_ref(
                    "u_filled",
                    ShaderUniform::Float(match filled {
                        true => 1.0,
                        false => 0.0,
                    }),
                );
                // todo: save slot shaders to cassette
                // if !filled && (slot.faction == Faction::Dark || slot.faction == Faction::Light) {
                //     shader.set_uniform_ref("u_scale", ShaderUniform::Float(0.0));
                // }
                shader.chain_after.clear();
                if filled && slot.faction == Faction::Shop {
                    shader
                        .chain_after
                        .push(options.shaders.slot_price.clone().set_uniform(
                            "u_color",
                            ShaderUniform::Color(
                                *options.colors.faction_colors.get(&Faction::Shop).unwrap(),
                            ),
                        ));
                }
            })
    }

    pub fn set_hovered_slot(world: &mut legion::World, faction: &Faction, slot: usize) {
        <(&SlotComponent, &mut Shader)>::query()
            .iter_mut(world)
            .for_each(|(slot_component, shader)| {
                shader.parameters.uniforms.insert(
                    "u_hovered".to_string(),
                    ShaderUniform::Float(
                        match slot_component.slot == slot && slot_component.faction == *faction {
                            true => 1.0,
                            false => 0.0,
                        },
                    ),
                )
            })
    }

    pub fn get_slot_filled_visual_effects(world: &legion::World) -> Vec<VisualEffect> {
        let filled_slots = Self::get_filled_slots(world);
        <(&SlotComponent, &EntityComponent)>::query()
            .iter(world)
            .map(|(slot, entity)| {
                VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityShaderConst {
                        entity: entity.entity,
                        uniforms: hashmap! {"u_filled" => ShaderUniform::Float(match filled_slots.contains(&(slot.faction, slot.slot)) {
                            true => 1.0,
                            false => 0.0,
                        })}
                        .into(),
                    },
                    0,
                )
            })
            .collect_vec()
    }

    pub fn make_gap(
        world: &mut legion::World,
        resources: &Resources,
        gap_slot: usize,
        factions: &HashSet<Faction>,
    ) -> bool {
        let mut current_slot: HashMap<&Faction, usize> =
            HashMap::from_iter(factions.iter().map(|faction| (faction, 0usize)));
        let mut changed = false;
        <(&mut UnitComponent, &EntityComponent)>::query()
            .iter_mut(world)
            .sorted_by_key(|(unit, _)| unit.slot)
            .for_each(|(unit, entity)| {
                if resources.input.cur_dragged.as_ref() != Some(&entity.entity) {
                    if let Some(slot) = current_slot.get_mut(&unit.faction) {
                        *slot = *slot + 1;
                        if *slot == gap_slot {
                            *slot = *slot + 1;
                        }
                        if unit.slot != *slot {
                            changed = true;
                            unit.slot = *slot;
                        }
                    }
                }
            });
        changed
    }

    pub fn fill_gaps(
        world: &mut legion::World,
        resources: &Resources,
        factions: &HashSet<Faction>,
    ) {
        Self::make_gap(world, resources, SLOTS_COUNT + 1, factions);
    }

    fn get_filled_slots(world: &legion::World) -> HashSet<(Faction, usize)> {
        HashSet::from_iter(
            <&UnitComponent>::query()
                .iter(world)
                .map(|unit| (unit.faction, unit.slot)),
        )
    }
}

impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &mut AreaComponent)>::query()
            .iter_mut(world)
            .for_each(|(unit, area)| {
                let need_pos = Self::get_unit_position(unit);
                area.position += (need_pos - area.position) * PULL_FORCE * resources.delta_time;
            })
    }
}
