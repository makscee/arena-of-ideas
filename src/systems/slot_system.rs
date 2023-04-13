use super::*;

pub struct SlotSystem {}

pub const MAX_SLOTS: usize = 8;
pub const PULL_FORCE: f32 = 7.0;
pub const SHOP_POSITION: vec2<f32> = vec2(-30.0, 0.0);
pub const BATTLEFIELD_POSITION: vec2<f32> = vec2(0.0, 0.0);
pub const SLOT_HOVER_DISTANCE: f32 = 3.0;

impl SlotSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_position(slot: usize, faction: &Faction, resources: &Resources) -> vec2<f32> {
        let floats = &resources.options.floats;
        let faction_mul: vec2<f32> = vec2(
            match faction {
                Faction::Light => -1.0,
                Faction::Dark => 1.0,
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
            Faction::Light | Faction::Dark => {
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
                SHOP_POSITION + floats.slots_shop_team_position + spacing * (slot + 2) as f32
            }
        }
    }

    pub fn get_scale(slot: usize, faction: Faction, resources: &Resources) -> f32 {
        match faction {
            Faction::Light | Faction::Dark => match slot == 1 {
                true => resources.options.floats.slots_striker_scale,
                false => resources.options.floats.slots_battle_team_scale,
            },
            _ => 1.0,
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
        mouse_pos: vec2<f32>,
        resources: &Resources,
    ) -> Option<(usize, Faction)> {
        let mut result = None;
        let mut min_distance = f32::MAX;
        for faction in Faction::all_iter() {
            for slot in 1..=MAX_SLOTS {
                let slot_pos = Self::get_position(slot, &faction, resources);
                let distance = (mouse_pos - slot_pos).len();
                if distance < SLOT_HOVER_DISTANCE && distance < min_distance {
                    result = Some((slot, faction));
                    min_distance = distance;
                }
            }
        }
        result
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

    pub fn create_entries(world: &mut legion::World, resources: &Resources) {
        for faction in Faction::all_iter() {
            for slot in 1..=MAX_SLOTS {
                Self::create_slot(faction, slot, world, resources);
            }
        }
    }

    fn create_slot(
        faction: Faction,
        slot: usize,
        world: &mut legion::World,
        resources: &Resources,
    ) {
        let entity = world.push((SlotComponent::new(slot, faction),));
        let position = Self::get_position(slot, &faction, resources);
        let color = faction.color(&resources.options);
        let scale = Self::get_scale(slot, faction, resources);
        let mut shader = resources.options.shaders.slot.clone();
        shader
            .parameters
            .uniforms
            .insert_color_ref("u_color", color)
            .insert_vec_ref("u_position", position)
            .insert_float_ref("u_scale", scale);
        match faction {
            Faction::Light => {}
            Faction::Dark => {}
            Faction::Team => {
                Self::add_slot_activation_btn(&mut shader, "Sell", None, entity, resources);
            }
            Faction::Shop => {
                shader
                    .chain_after
                    .push(resources.options.shaders.slot_price.clone().set_uniform(
                        "u_text",
                        ShaderUniform::String((
                            0,
                            format!("{} g", ShopSystem::buy_price(resources)),
                        )),
                    ));
            }
        };

        let mut entry = world.entry(entity).unwrap();
        entry.add_component(TapeEntityComponent::new(entity));
        entry.add_component(shader);
    }

    fn add_slot_activation_btn(
        shader: &mut Shader,
        text: &str,
        icon: Option<Image>,
        entity: legion::Entity,
        resources: &Resources,
    ) {
        let mut button = ButtonSystem::create_button(
            Some(text),
            icon,
            Self::activation_handler,
            entity,
            &resources.options,
        );
        button
            .parameters
            .uniforms
            .add_mapping("u_enabled", "u_filled");
        shader.chain_after.push(button.set_uniform(
            "u_offset",
            ShaderUniform::Vec2(vec2(0.0, -resources.options.floats.slot_info_offset)),
        ));
    }

    fn activation_handler(
        event: InputEvent,
        entity: legion::Entity,
        _: &mut Shader,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        match event {
            InputEvent::Click => {
                let slot = world
                    .entry(entity)
                    .unwrap()
                    .get_component::<SlotComponent>()
                    .unwrap()
                    .clone();
                Self::handle_slot_activation(slot.slot, slot.faction, world, resources);
            }
            _ => {}
        }
    }

    fn handle_slot_activation(
        slot: usize,
        faction: Faction,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        match faction {
            Faction::Team => {
                if let Some(entity) = Self::find_unit_by_slot(slot, &faction, world, resources) {
                    ShopSystem::try_sell(entity, resources, world);
                }
            }
            _ => {}
        }
    }

    pub fn handle_unit_drag(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        if let Some((slot, faction)) = Self::get_hovered_slot(resources.input.mouse_pos, resources)
        {
            if faction == Faction::Team {
                Self::make_gap(faction, slot, world, resources, Some(entity));
            }
        }
    }

    pub fn handle_unit_drop(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        if let Some((slot, faction)) = Self::get_hovered_slot(resources.input.mouse_pos, resources)
        {
            let unit_faction = UnitSystem::get_unit(entity, world).faction;
            if faction == Faction::Team && unit_faction == Faction::Shop {
                ShopSystem::try_buy(entity, slot, resources, world);
            } else if faction == Faction::Shop && unit_faction == Faction::Team {
                ShopSystem::try_sell(entity, resources, world);
            } else if faction == unit_faction {
                if Self::make_gap(faction, slot, world, resources, Some(entity)) {
                    UnitSystem::set_slot(entity, slot, world);
                }
            }
        }
    }

    pub fn fill_gaps(faction: Faction, world: &mut legion::World) {
        let mut slot = 1;
        <&mut UnitComponent>::query()
            .iter_mut(world)
            .filter(|x| x.faction == faction)
            .sorted_by_key(|x| x.slot)
            .for_each(|x| {
                x.slot = slot;
                slot += 1;
            })
    }

    pub fn make_gap(
        faction: Faction,
        slot: usize,
        world: &mut legion::World,
        resources: &mut Resources,
        ignore: Option<legion::Entity>,
    ) -> bool {
        let slots = resources.team_states.get_slots(&faction);
        let mut units = UnitSystem::collect_faction_units(world, resources, faction, false);
        if let Some(ignore) = ignore {
            units.remove(&ignore);
        }
        if units.len() >= slots {
            return false;
        }
        let mut units_slots: Vec<Option<legion::Entity>> = vec![None; slots + 1];
        units
            .into_iter()
            .for_each(|(entity, unit)| units_slots[unit.slot] = Some(entity));
        if units_slots[slot].is_none() {
            return true;
        }
        for empty in (1..slot).rev() {
            if units_slots[empty].is_none() {
                for i in empty..slot {
                    UnitSystem::set_slot(units_slots[i + 1].unwrap(), i, world);
                }
                return true;
            }
        }
        for empty in (slot + 1)..=slots {
            if units_slots[empty].is_none() {
                for i in slot..empty {
                    UnitSystem::set_slot(units_slots[i].unwrap(), i + 1, world);
                }
                return true;
            }
        }
        panic!("Failed to make a slot gap")
    }

    pub fn refresh_slots(
        factions: &HashSet<Faction>,
        units: &HashMap<legion::Entity, UnitComponent>,
        world: &mut legion::World,
        resources: &Resources,
    ) {
        let filled: HashSet<(Faction, usize)> =
            HashSet::from_iter(units.iter().map(|(_, unit)| (unit.faction, unit.slot)));
        for (slot, shader) in <(&SlotComponent, &mut Shader)>::query().iter_mut(world) {
            if !factions.contains(&slot.faction) {
                continue;
            }
            let filled = filled.contains(&(slot.faction, slot.slot)) as i32 as f32;
            let enabled =
                (slot.slot <= resources.team_states.get_slots(&slot.faction)) as i32 as f32;
            shader
                .parameters
                .uniforms
                .insert_float_ref("u_filled", filled)
                .insert_float_ref("u_enabled", enabled);
        }
    }
}

impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &mut AreaComponent, &EntityComponent)>::query()
            .iter_mut(world)
            .for_each(|(unit, area, entity)| {
                if resources.input.frame_data.1.is_dragged(entity.entity) {
                    return;
                }
                let need_pos = Self::get_unit_position(unit, resources);
                area.position += (need_pos - area.position) * PULL_FORCE * resources.delta_time;
            })
    }
}
