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
                Faction::Dark | Faction::Team | Faction::Shop => 1.0,
            },
            1.0,
        );
        let spacing: vec2<f32> = match faction {
            Faction::Dark | Faction::Light => floats.slots_battle_team_spacing,
            Faction::Team | Faction::Shop => floats.slots_shop_spacing,
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
            if let Some(state) = TeamSystem::try_get_state(&faction, world) {
                for slot in 1..=state.vars.get_int(&VarName::Slots) as usize {
                    let slot_pos = Self::get_position(slot, &faction, resources);
                    let distance = (mouse_pos - slot_pos).len();
                    if distance < SLOT_HOVER_DISTANCE && distance < min_distance {
                        result = Some((slot, faction));
                        min_distance = distance;
                    }
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
        HashMap::from_iter(factions.into_iter().filter_map(|faction| {
            if let Some(state) = TeamSystem::try_get_state(&faction, world) {
                Some((faction, state.vars.get_int(&VarName::Slots) as usize))
            } else {
                None
            }
        }))
    }

    pub fn create_entries(world: &mut legion::World, resources: &Resources) {
        for faction in Faction::all_iter() {
            for slot in 1..=MAX_SLOTS {
                Self::create_slot(faction, slot, world, resources);
            }
        }
    }

    pub fn get_slot_entity(slot: usize, faction: Faction, world: &legion::World) -> legion::Entity {
        let result = <(&EntityComponent, &SlotComponent)>::query()
            .iter(world)
            .find_map(|(entity, s)| match s.slot == slot {
                true => Some(entity.entity),
                false => None,
            });
        result.expect(&format!("Failed to find slot {slot} {faction}"))
    }

    fn create_slot(
        faction: Faction,
        slot: usize,
        world: &mut legion::World,
        resources: &Resources,
    ) {
        if faction == Faction::Shop {
            return;
        }
        let entity = world.push((SlotComponent::new(slot, faction), TapeEntityComponent {}));
        let position = Self::get_position(slot, &faction, resources);
        let color = faction.color(&resources.options);
        let scale = Self::get_scale(slot, faction, resources);
        let mut shader = resources.options.shaders.slot.clone();
        shader
            .parameters
            .uniforms
            .insert_color_ref("u_color".to_owned(), color)
            .insert_vec2_ref("u_position".to_owned(), position)
            .insert_float_ref("u_scale".to_owned(), scale);

        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent::new(entity));
        entry.add_component(shader);
    }

    pub fn clear_slots_buttons(faction: Faction, world: &mut legion::World) {
        for (slot, shader) in <(&SlotComponent, &mut Shader)>::query().iter_mut(world) {
            if slot.faction != faction {
                continue;
            }
            shader.chain_after.clear();
        }
    }

    pub fn add_slots_buttons(
        faction: Faction,
        text: &str,
        enable_key: Option<&str>,
        activate_key: Option<&str>,
        world: &mut legion::World,
        resources: &Resources,
    ) {
        for (slot, entity, shader) in
            <(&SlotComponent, &EntityComponent, &mut Shader)>::query().iter_mut(world)
        {
            if slot.faction != faction {
                continue;
            }
            Self::add_slot_activation_btn(
                shader,
                text,
                enable_key,
                activate_key,
                entity.entity,
                resources,
            );
        }
    }

    fn add_slot_activation_btn(
        shader: &mut Shader,
        text: &str,
        enable_key: Option<&str>,
        activate_key: Option<&str>,
        entity: legion::Entity,
        resources: &Resources,
    ) {
        let mut button = ButtonSystem::create_button(
            Some(text),
            Self::activation_handler,
            None,
            entity,
            Some(resources.options.shaders.slot_button.clone()),
            default(),
            &resources.options,
        );

        if let Some(key) = enable_key {
            button.parameters.uniforms.add_mapping("u_enabled", key);
        }
        if let Some(key) = activate_key {
            button.parameters.uniforms.add_mapping("u_active", key);
        }
        shader.chain_after.push(button.insert_uniform(
            "u_offset".to_owned(),
            ShaderUniform::Vec2(vec2(0.0, -resources.options.floats.slot_info_offset)),
        ));
    }

    fn activation_handler(
        event: HandleEvent,
        entity: legion::Entity,
        _: &mut Shader,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        match event {
            HandleEvent::Click => {
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

    pub fn handle_state_enter(
        state: GameState,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        for (slot, entity, shader) in
            <(&SlotComponent, &EntityComponent, &mut Shader)>::query().iter_mut(world)
        {
            shader.chain_after.clear();
            let entity = entity.entity;
            let faction = slot.faction;
            let enabled = match state {
                GameState::Battle => faction == Faction::Dark || faction == Faction::Light,
                GameState::Sacrifice => {
                    if faction == Faction::Team {
                        Self::add_slot_activation_btn(
                            shader,
                            "Sacrifice",
                            Some("u_filled"),
                            None,
                            entity,
                            resources,
                        )
                    }

                    faction == Faction::Team
                }
                _ => true,
            };
            shader.insert_float_ref("u_enabled".to_owned(), enabled as i32 as f32);
        }
    }

    fn handle_slot_activation(
        slot: usize,
        faction: Faction,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        if let Some(entity) = Self::find_unit_by_slot(slot, &faction, world) {
            match faction {
                Faction::Team => match resources.current_state {
                    GameState::Shop => {
                        if let Some(data) = resources.shop_data.status_apply.as_mut() {
                            data.2 = StatusTarget::Single { slot: Some(slot) };
                        }
                        ShopSystem::finish_status_apply(world, resources);
                    }
                    GameState::Sacrifice => {
                        if !resources.sacrifice_data.marked_units.contains(&entity) {
                            resources.sacrifice_data.marked_units.insert(entity);
                            debug!("Mark {entity:?}");
                            let position = Self::get_position(slot, &faction, resources);
                            let text = format!(
                                "+{}",
                                ContextState::get(entity, world).get_int(&VarName::Rank, world)
                            );
                            Node::new_panel_scaled(
                                resources
                                    .options
                                    .shaders
                                    .slot_sacrifice_marker
                                    .clone()
                                    .insert_vec2("u_position".to_owned(), position)
                                    .insert_color(
                                        "u_color".to_owned(),
                                        resources.options.colors.subtract,
                                    )
                                    .insert_string("u_star_text".to_owned(), text, 1),
                            )
                            .lock(NodeLockType::Empty)
                            .push_as_panel(entity, resources);
                        } else {
                            resources
                                .tape_player
                                .tape
                                .close_panels(entity, resources.tape_player.head);
                            resources.sacrifice_data.marked_units.remove(&entity);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
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
                Self::make_gap(faction, slot, world, Some(entity));
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
            if faction == unit_faction {
                if Self::make_gap(faction, slot, world, Some(entity)) {
                    let state = ContextState::get_mut(entity, world);
                    state.vars.set_int(&VarName::Slot, slot as i32);
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
        ignore: Option<legion::Entity>,
    ) -> bool {
        let slots = TeamSystem::get_state(&faction, world)
            .vars
            .get_int(&VarName::Slots) as usize;
        let units = UnitSystem::collect_faction(world, faction)
            .into_iter()
            .filter(|entity| Some(*entity) != ignore)
            .collect_vec();
        if units.len() >= slots {
            return false;
        }
        let mut units_slots: Vec<Option<legion::Entity>> = vec![None; slots + 1];
        units.into_iter().for_each(|entity| {
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
                .insert_float_ref("u_filled".to_owned(), filled)
                .insert_float_ref("u_enabled".to_owned(), enabled)
                .insert_float_ref("u_hovered".to_owned(), hovered);
        }
    }
}

const DELTA_THRESHOLD: f32 = 0.01;
impl System for SlotSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        for entity in UnitSystem::all(world) {
            if resources.input_data.frame_data.1.is_dragged(entity) {
                continue;
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
