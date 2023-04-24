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
                Faction::Dark | Faction::Team | Faction::Shop | Faction::Sacrifice => 1.0,
            },
            1.0,
        );
        let spacing: vec2<f32> = match faction {
            Faction::Dark | Faction::Light => floats.slots_battle_team_spacing,
            Faction::Team | Faction::Shop => floats.slots_shop_spacing,
            Faction::Sacrifice => floats.slots_sacrifice_spacing,
        };
        let slot = slot as i32 - 1;
        match faction {
            Faction::Light | Faction::Dark => {
                return match slot == 0 {
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
                SHOP_POSITION + floats.slots_shop_team_position + spacing * (slot + 1) as f32
            }
            Faction::Sacrifice => {
                SHOP_POSITION + floats.slots_sacrifice_position + spacing * slot as f32
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

    pub fn move_to_slots_animated(
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) {
        if let Some(cluster) = cluster {
            let mut node = Node::default();
            <(&EntityComponent, &ContextState)>::query()
                .filter(component::<UnitComponent>())
                .iter(world)
                .filter_map(|(entity, state)| {
                    let slot = state.vars.get_int(&VarName::Slot) as usize;
                    let faction = state.get_faction(&VarName::Faction, world);
                    let need_pos = Self::get_position(slot, &faction, resources);
                    let pos = state.vars.get_vec2(&VarName::Position);
                    match (need_pos - pos).len() < 0.0001 {
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
                        resources,
                        EasingType::QuartInOut,
                        0.8,
                    )
                });
            cluster.push(node.lock(NodeLockType::Full { world, resources }));
        }
    }

    pub fn get_hovered_slot(
        mouse_pos: vec2<f32>,
        world: &legion::World,
        resources: &Resources,
    ) -> Option<(usize, Faction)> {
        let mut result = None;
        let mut min_distance = f32::MAX;
        for faction in Faction::all_iter() {
            for slot in 1..=TeamSystem::get_state(&faction, world)
                .vars
                .get_int(&VarName::Slots) as usize
            {
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
    ) -> Option<legion::Entity> {
        if slot
            > TeamSystem::get_state(faction, world)
                .vars
                .get_int(&VarName::Slots) as usize
        {
            return None;
        }
        <(&EntityComponent, &ContextState)>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
            .find_map(|(entity, state)| {
                match state.get_faction(&VarName::Faction, world) == *faction
                    && state.vars.get_int(&VarName::Slot) as usize == slot
                {
                    true => Some(entity.entity),
                    false => None,
                }
            })
    }

    pub fn get_factions_slots(
        factions: HashSet<Faction>,
        world: &legion::World,
    ) -> HashMap<Faction, usize> {
        HashMap::from_iter(factions.into_iter().map(|faction| {
            (
                faction,
                TeamSystem::get_state(&faction, world)
                    .vars
                    .get_int(&VarName::Slots) as usize,
            )
        }))
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
        let entity = world.push((SlotComponent::new(slot, faction), TapeEntityComponent {}));
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
            Faction::Light | Faction::Dark => {}
            Faction::Team => {
                Self::add_slot_activation_btn(&mut shader, "Sell", None, entity, resources);
            }
            Faction::Sacrifice => {
                if slot == 1 {
                    Self::add_slot_activation_btn(
                        &mut shader,
                        "Sacrifice",
                        None,
                        entity,
                        resources,
                    );
                }
            }
            Faction::Shop => {
                shader
                    .chain_after
                    .push(resources.options.shaders.slot_price.clone().set_uniform(
                        "u_text",
                        ShaderUniform::String((0, format!("{} g", ShopSystem::buy_price(world)))),
                    ));
            }
        };

        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent::new(entity));
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
                if let Some(entity) = Self::find_unit_by_slot(slot, &faction, world) {
                    ShopSystem::try_sell(entity, resources, world);
                }
            }
            Faction::Sacrifice => {
                let units = UnitSystem::collect_faction(world, faction);
                BonusEffectPool::load_widget(units.len(), world, resources);
                for unit in units {
                    world.remove(unit);
                }
                TeamSystem::get_state_mut(&faction, world)
                    .vars
                    .set_int(&VarName::Slots, 1);
            }
            _ => {}
        }
    }

    pub fn handle_unit_drag(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        if let Some((slot, faction)) =
            Self::get_hovered_slot(resources.input_data.mouse_world_pos, world, resources)
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
        if let Some((slot, faction)) =
            Self::get_hovered_slot(resources.input_data.mouse_world_pos, world, resources)
        {
            let state = ContextState::get(entity, world);
            let unit_faction = state.get_faction(&VarName::Faction, world);
            if faction == Faction::Team && unit_faction == Faction::Shop {
                ShopSystem::try_buy(entity, slot, resources, world);
            } else if faction == Faction::Shop && unit_faction == Faction::Team {
                ShopSystem::try_sell(entity, resources, world);
            } else if faction == unit_faction {
                if Self::make_gap(faction, slot, world, resources, Some(entity)) {
                    let state = ContextState::get_mut(entity, world);
                    state.vars.set_int(&VarName::Slot, slot as i32);
                }
            } else if faction == Faction::Sacrifice && unit_faction == Faction::Team
                || faction == Faction::Team && unit_faction == Faction::Sacrifice
            {
                if SlotSystem::find_unit_by_slot(slot, &faction, world).is_none() {
                    let state = ContextState::get_mut(entity, world);
                    state.vars.set_faction(&VarName::Faction, faction);
                    state.vars.set_int(&VarName::Slot, slot as i32);

                    let units = UnitSystem::collect_faction(world, Faction::Sacrifice).len() as i32;
                    TeamSystem::get_state_mut(&Faction::Sacrifice, world)
                        .vars
                        .set_int(&VarName::Slots, units + 1);
                }
            }
        }
    }

    pub fn fill_gaps(faction: Faction, world: &mut legion::World) {
        UnitSystem::collect_faction_states(world, faction)
            .into_iter()
            .sorted_by_key(|(_, state)| state.vars.get_int(&VarName::Slot))
            .map(|(entity, _)| entity)
            .collect_vec()
            .into_iter()
            .enumerate()
            .for_each(|(i, entity)| {
                ContextState::get_mut(entity, world)
                    .vars
                    .set_int(&VarName::Slot, i as i32 + 1);
            });
    }

    pub fn make_gap(
        faction: Faction,
        slot: usize,
        world: &mut legion::World,
        resources: &mut Resources,
        ignore: Option<legion::Entity>,
    ) -> bool {
        let slots = TeamSystem::get_state(&faction, world)
            .vars
            .get_int(&VarName::Slots) as usize;
        let units = UnitSystem::collect_faction(world, faction);
        if units.len() >= slots {
            return false;
        }
        let mut units_slots: Vec<Option<legion::Entity>> = vec![None; slots + 1];
        units.into_iter().for_each(|entity| {
            if ignore == Some(entity) {
                return;
            }
            let slot = ContextState::get(entity, world)
                .vars
                .get_int(&VarName::Slot) as usize;
            units_slots[slot] = Some(entity);
        });
        if units_slots[slot].is_none() {
            return true;
        }
        for empty in (1..slot).rev() {
            if units_slots[empty].is_none() {
                for i in empty..slot {
                    ContextState::get_mut(units_slots[i + 1].unwrap(), world)
                        .vars
                        .set_int(&VarName::Slot, i as i32);
                }
                return true;
            }
        }
        for empty in (slot + 1)..=slots {
            if units_slots[empty].is_none() {
                for i in slot..empty {
                    ContextState::get_mut(units_slots[i].unwrap(), world)
                        .vars
                        .set_int(&VarName::Slot, i as i32 + 1);
                }
                return true;
            }
        }
        panic!("Failed to make a slot gap")
    }

    pub fn refresh_slots(
        factions: HashSet<Faction>,
        world: &mut legion::World,
        resources: &Resources,
    ) {
        let filled: HashSet<(Faction, usize)> = HashSet::from_iter(
            UnitSystem::collect_factions(world, &factions)
                .into_iter()
                .map(|entity| {
                    let state = ContextState::get(entity, world);
                    (
                        state.get_faction(&VarName::Faction, world),
                        state.get_int(&VarName::Slot, world) as usize,
                    )
                }),
        );
        let mut hovered = None;
        if let Some(dragged) = resources.input_data.dragged_entity {
            hovered =
                Self::get_hovered_slot(resources.input_data.mouse_world_pos, world, resources);
        }
        let factions = Self::get_factions_slots(factions, world);
        for (slot, shader) in <(&SlotComponent, &mut Shader)>::query().iter_mut(world) {
            if !factions.contains_key(&slot.faction) {
                continue;
            }
            let slots = *factions.get(&slot.faction).unwrap();
            let filled = filled.contains(&(slot.faction, slot.slot)) as i32 as f32;
            let enabled = (slot.slot <= slots) as i32 as f32;
            let hovered = if let Some(hovered) = hovered {
                ((slot.slot, slot.faction) == hovered) as i32 as f32
            } else {
                0.0
            };

            shader
                .parameters
                .uniforms
                .insert_float_ref("u_filled", filled)
                .insert_float_ref("u_enabled", enabled)
                .insert_float_ref("u_hovered", hovered);
        }
    }
}

const DELTA_THRESHOLD: f32 = 0.01;
impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        for entity in UnitSystem::all(world) {
            if resources.input_data.frame_data.1.is_dragged(entity) {
                return;
            }
            let (position, slot, faction) = {
                let context = Context::new(ContextLayer::Unit { entity }, world, resources);
                (
                    context.get_vec2(&VarName::Position, world).unwrap(),
                    context.get_int(&VarName::Slot, world).unwrap() as usize,
                    context.get_faction(&VarName::Faction, world).unwrap(),
                )
            };
            let delta = Self::get_position(slot, &faction, &resources) - position;
            if delta.len() > DELTA_THRESHOLD {
                ContextState::get_mut(entity, world).vars.change_vec2(
                    &VarName::Position,
                    delta * PULL_FORCE * resources.delta_time,
                );
            }
        }
    }
}
