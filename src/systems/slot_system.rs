use super::*;

pub struct SlotSystem {}

pub const SLOTS_COUNT: usize = 5;

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
            Faction::Team | Faction::Shop => vec2(slot as f32, 0.0) * faction_mul * 2.5,
        }
    }

    pub fn get_unit_position(unit: &UnitComponent) -> vec2<f32> {
        Self::get_position(unit.slot, &unit.faction)
    }

    fn clear_world(world: &mut legion::World) {
        <(&SlotComponent, &EntityComponent)>::query()
            .iter(world)
            .map(|(_, entity)| entity.entity)
            .collect_vec()
            .iter()
            .for_each(|entity| {
                world.remove(*entity);
            });
    }

    pub fn add_slot_shaders(
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
                    Position(Self::get_position(slot, faction)),
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
            })
        }
    }
}

impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {}
}
