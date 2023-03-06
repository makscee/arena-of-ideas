use super::*;

pub struct SlotSystem {}

pub const SLOTS_COUNT: usize = 5;
pub const PULL_FORCE: f32 = 7.0;
pub const SHOP_POSITION: vec2<f32> = vec2(0.0, 5.0);

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
                Faction::Shop => 1.0,
                Faction::Gallery => 0.0,
            },
            1.0,
        );
        match faction {
            Faction::Light | Faction::Dark => {
                return match slot == 1 {
                    true => vec2(1.5, 0.0),
                    false => vec2((slot as f32 - 1.0) * 2.5, -4.0),
                } * faction_mul
            }
            Faction::Team | Faction::Gallery => vec2(slot as f32, 0.0) * faction_mul * 2.5,
            Faction::Shop => {
                SHOP_POSITION
                    + vec2(slot as f32 - SLOTS_COUNT as f32 / 2.0, 0.0) * faction_mul * 2.5
            }
        }
    }

    pub fn get_unit_position(unit: &UnitComponent) -> vec2<f32> {
        Self::get_position(unit.slot, &unit.faction)
    }

    pub fn get_hovered_slot(faction: &Faction, mouse_pos: vec2<f32>) -> Option<usize> {
        for slot in 1..=SLOTS_COUNT {
            let slot_pos = Self::get_position(slot, faction);
            if (slot_pos - mouse_pos).len() < DEFAULT_UNIT_RADIUS {
                return Some(slot);
            }
        }
        None
    }

    pub fn get_horizontal_hovered_slot(faction: &Faction, mouse_pos: vec2<f32>) -> Option<usize> {
        for slot in 1..=SLOTS_COUNT {
            let slot_pos = Self::get_position(slot, faction);
            if (mouse_pos.x - slot_pos.x).abs() < DEFAULT_UNIT_RADIUS {
                return Some(slot);
            }
        }
        None
    }

    pub fn find_unit_by_slot(
        slot: usize,
        faction: &Faction,
        world: &legion::World,
    ) -> Option<(legion::Entity, UnitComponent)> {
        <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .find_map(
                |(entity, unit)| match unit.faction == *faction && unit.slot == slot {
                    true => Some((entity.entity, *unit)),
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

    pub fn refresh_slot_shaders(
        world: &mut legion::World,
        resources: &Resources,
        factions: HashSet<Faction>,
    ) {
        Self::clear_world(world);
        let filled_slots: HashSet<(Faction, usize)> = HashSet::from_iter(
            <&UnitComponent>::query()
                .iter(world)
                .map(|unit| (unit.faction, unit.slot)),
        );
        for slot in 1..=SLOTS_COUNT {
            factions.iter().for_each(|faction| {
                let entity = world.push((
                    PositionComponent(Self::get_position(slot, faction)),
                    resources
                        .options
                        .slot
                        .clone()
                        .set_uniform(
                            "u_color",
                            ShaderUniform::Color(faction.color(&resources.options)),
                        )
                        .set_uniform(
                            "u_filled",
                            ShaderUniform::Float(match filled_slots.contains(&(*faction, slot)) {
                                true => 1.0,
                                false => 0.0,
                            }),
                        ),
                    SlotComponent { slot },
                ));
                world
                    .entry(entity)
                    .unwrap()
                    .add_component(EntityComponent { entity });
                let context = Context {
                    owner: entity,
                    target: entity,
                    parent: Some(WorldSystem::get_context(world).owner),
                    vars: default(),
                };
                world.entry(entity).unwrap().add_component(context);
            })
        }
    }

    pub fn make_gap(world: &mut legion::World, gap_slot: usize, factions: HashSet<Faction>) {
        let mut current_slot: HashMap<&Faction, usize> =
            HashMap::from_iter(factions.iter().map(|faction| (faction, 0usize)));
        <&mut UnitComponent>::query()
            .filter(!component::<DragComponent>())
            .iter_mut(world)
            .sorted_by_key(|unit| unit.slot)
            .for_each(|unit| {
                if let Some(slot) = current_slot.get_mut(&unit.faction) {
                    *slot = *slot + 1;
                    if *slot == gap_slot {
                        *slot = *slot + 1;
                    }
                    unit.slot = *slot;
                }
            });
    }

    pub fn fill_gaps(world: &mut legion::World, factions: HashSet<Faction>) {
        Self::make_gap(world, SLOTS_COUNT + 1, factions);
    }
}

impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &mut PositionComponent)>::query()
            .iter_mut(world)
            .for_each(|(unit, position)| {
                let need_pos = Self::get_unit_position(unit);
                position.0 += (need_pos - position.0) * PULL_FORCE * resources.delta_time;
            })
    }
}
