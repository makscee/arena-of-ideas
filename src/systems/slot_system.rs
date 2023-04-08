use super::*;

pub struct SlotSystem {}

pub const MAX_SLOTS: usize = 8;
pub const PULL_FORCE: f32 = 7.0;
pub const SHOP_POSITION: vec2<f32> = vec2(-30.0, 0.0);
pub const BATTLEFIELD_POSITION: vec2<f32> = vec2(0.0, 0.0);

impl SlotSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_position(slot: usize, faction: &Faction, resources: &Resources) -> vec2<f32> {
        let floats = &resources.options.floats;
        let faction_mul: vec2<f32> = vec2(
            match faction {
                Faction::Light => -1.0,
                Faction::Dark { .. } => 1.0,
                Faction::Team => 1.0,
                Faction::Shop => 1.0,
            },
            1.0,
        );
        let spacing: vec2<f32> = match faction {
            Faction::Dark | Faction::Light => floats.slots_battle_team_spacing,
            Faction::Team | Faction::Shop => floats.slots_shop_spacing,
        };
        match faction {
            Faction::Light | Faction::Dark { .. } => {
                return match slot == 1 {
                    true => floats.slots_striker_position,
                    false => floats.slots_battle_team_position + spacing * slot as f32,
                } * faction_mul
                    + BATTLEFIELD_POSITION
            }
            Faction::Team => {
                SHOP_POSITION
                    + floats.slots_shop_team_position * vec2(1.0, -1.0)
                    + spacing * slot as f32
            }
            Faction::Shop => {
                let offset = (MAX_SLOTS - resources.team_states.get_slots(faction)) / 2;
                SHOP_POSITION + floats.slots_shop_team_position + spacing * (slot + offset) as f32
            }
        }
    }

    pub fn get_scale(slot: i32, faction: f32, resources: &Resources) -> f32 {
        if faction == Faction::Dark.float_value() || faction == Faction::Light.float_value() {
            match slot == 1 {
                true => resources.options.floats.slots_striker_scale,
                false => resources.options.floats.slots_battle_team_scale,
            }
        } else {
            1.0
        }
    }

    pub fn get_unit_position(unit: &UnitComponent, resources: &Resources) -> vec2<f32> {
        Self::get_position(unit.slot, &unit.faction, resources)
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
                    let need_pos = Self::get_unit_position(unit, resources);
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

    pub fn get_hovered_slot(
        faction: &Faction,
        mouse_pos: vec2<f32>,
        resources: &Resources,
    ) -> Option<usize> {
        for slot in 1..=MAX_SLOTS {
            let slot_pos = Self::get_position(slot, faction, resources);
            if (mouse_pos - slot_pos).len() < 1.5 {
                return Some(slot);
            }
        }
        None
    }

    pub fn find_unit_by_slot(
        slot: usize,
        faction: &Faction,
        world: &legion::World,
        resources: &Resources,
    ) -> Option<legion::Entity> {
        if slot > resources.team_states.get_team_state(faction).slots {
            return None;
        }
        <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .find_map(|(entity, unit)| {
                match unit.faction == *faction
                    && unit.slot == slot
                    && UnitSystem::unit_on_field(unit, resources)
                {
                    true => Some(entity.entity),
                    false => None,
                }
            })
    }

    pub fn draw_slots_to_node(
        node: &mut Node,
        factions: &HashSet<Faction>,
        units: &HashMap<legion::Entity, UnitComponent>,
        resources: &Resources,
    ) {
        let filled_slots: HashSet<(Faction, usize)> =
            HashSet::from_iter(units.values().map(|x| (x.faction, x.slot)));
        for faction in factions {
            for slot in 1..=resources.team_states.get_team_state(faction).slots {
                let filled = filled_slots.contains(&(*faction, slot));
                node.add_effect_by_key(
                    "slots".to_string(),
                    VisualEffect::new(
                        0.0,
                        VisualEffectType::ShaderConst {
                            shader: Self::get_shader(slot, faction, filled, resources),
                        },
                        0,
                    ),
                )
            }
        }
    }

    fn get_shader(slot: usize, faction: &Faction, filled: bool, resources: &Resources) -> Shader {
        resources
            .options
            .shaders
            .slot
            .clone()
            .set_uniform(
                "u_color",
                ShaderUniform::Color(faction.color(&resources.options)),
            )
            .set_uniform(
                "u_position",
                ShaderUniform::Vec2(Self::get_position(slot, faction, resources)),
            )
            .set_uniform(
                "u_scale",
                ShaderUniform::Float(Self::get_scale(
                    slot as i32,
                    faction.float_value(),
                    resources,
                )),
            )
            .set_uniform("u_filled", ShaderUniform::Float(filled as i32 as f32))
            .set_uniform(
                "u_hovered",
                ShaderUniform::Float(Self::is_slot_hovered(faction, slot, resources) as i32 as f32),
            )
    }

    pub fn set_hovered_slot(faction: Faction, slot: usize, resources: &mut Resources) {
        resources.input.hovered_slot = Some((faction, slot));
    }

    pub fn reset_hovered_slot(resources: &mut Resources) {
        resources.input.hovered_slot = None;
    }

    fn is_slot_hovered(faction: &Faction, slot: usize, resources: &Resources) -> bool {
        if let Some(data) = resources.input.hovered_slot {
            data.0 == *faction && slot == data.1
        } else {
            false
        }
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
        Self::make_gap(world, resources, MAX_SLOTS + 1, factions);
    }
}

impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &mut AreaComponent)>::query()
            .iter_mut(world)
            .for_each(|(unit, area)| {
                let need_pos = Self::get_unit_position(unit, resources);
                area.position += (need_pos - area.position) * PULL_FORCE * resources.delta_time;
            })
    }
}
