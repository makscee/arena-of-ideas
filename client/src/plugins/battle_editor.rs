use super::*;

pub struct BattleEditorPlugin;

impl Plugin for BattleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleEditorState>()
            .add_systems(OnEnter(GameState::Loaded), Self::init);
    }
}

#[derive(Resource)]
pub struct BattleEditorState {
    pub current_node: Option<BattleEditorNode>,
    pub navigation_stack: Vec<BattleEditorNode>,
    pub is_left_team: bool,
}

impl Default for BattleEditorState {
    fn default() -> Self {
        Self {
            current_node: None,
            navigation_stack: Vec::new(),
            is_left_team: true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BattleEditorNode {
    Team(u64),
    House(u64),
    Unit(u64),
}

impl BattleEditorPlugin {
    fn init(world: &mut World) {
        let mut state = world.resource_mut::<BattleEditorState>();
        if state.current_node.is_none() {
            let id = pd().client_state.battle_test.0.root;
            if id > 0 {
                state.current_node = Some(BattleEditorNode::Team(id));
            }
        }
    }

    // Generic helper for managing node parts
    fn manage_part<T: FPlaceholder + FEdit + ClientNode>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        id: u64,
    ) -> NodeResult<(u64, bool)> {
        let mut changed = false;
        let mut part_id = 0;
        let entity = ctx.entity(id)?;
        let part_result = ctx.load::<T>(id);

        match part_result {
            Ok(part) => {
                part_id = part.id();
                let part_entity = part.entity(ctx)?;
                let mut part = part.clone();

                ui.group(|ui| {
                    ui.label(format!("{}:", part.kind()));
                    if part.edit(ctx, ui).changed() {
                        part.clone().spawn(ctx, Some(part_entity)).log();
                        changed = true;
                    }
                    if "[red Delete]".cstr().button(ui).clicked() {
                        ctx.despawn(part.id()).log();
                        changed = true;
                    }
                });
            }
            Err(_) => {
                if ui.button(format!("➕ Add {}", T::kind_s())).clicked() {
                    let part = T::placeholder(next_id());
                    part_id = part.id();
                    let part_entity = ctx.world_mut()?.spawn_empty().id();
                    part.spawn(ctx, Some(part_entity))?;

                    ctx.add_link_entities(entity, part_entity)?;
                    changed = true;
                }
            }
        }

        Ok((part_id, changed))
    }

    // Generic helper for managing collections of child nodes
    fn manage_parts<T: FPlaceholder + FTitle + FCopy + FPaste + ClientNode>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        parent_entity: Entity,
        collection_name: &str,
        get_children_fn: impl Fn() -> NodeResult<Vec<T>>,
        on_edit: impl Fn(T) -> Option<BattleEditorAction>,
    ) -> NodeResult<(bool, Option<BattleEditorAction>)> {
        let mut changed = false;
        let mut navigation_action = None;

        ui.label(format!("{}:", collection_name));
        let children = get_children_fn()?;
        let mut children_to_delete = Vec::new();
        let mut children_to_replace = Vec::new();

        for child in children {
            ui.horizontal(|ui| -> NodeResult<()> {
                child.title(ctx).label(ui);

                if ui.button("Edit").clicked() {
                    navigation_action = on_edit(child.clone());
                }

                if ui.button("📋 Copy").clicked() {
                    child.copy_to_clipboard();
                }

                if ui.button("📋 Paste").clicked() {
                    if let Some(replacement) = T::paste_from_clipboard() {
                        children_to_replace.push((child.entity(ctx)?, replacement));
                    }
                }

                if ui.button("🗑 Delete").clicked() {
                    children_to_delete.push(child.entity(ctx)?);
                }
                Ok(())
            });
        }

        for (entity, replacement) in children_to_replace {
            if let Err(e) = replacement.spawn(ctx, Some(entity)) {
                error!("Failed to replace {}: {}", collection_name, e);
            } else {
                changed = true;
            }
        }

        for entity in children_to_delete {
            ctx.despawn(ctx.id(entity)?).log();
            changed = true;
        }

        if ui.button(format!("➕ Add {}", collection_name)).clicked() {
            let item = T::placeholder(next_id());
            let item_entity = ctx.world_mut()?.spawn_empty().id();
            item.spawn(ctx, Some(item_entity))?;
            ctx.add_link_entities(parent_entity, item_entity)
                .notify_error(ctx.world_mut()?);
            changed = true;
        }

        Ok((changed, navigation_action))
    }

    pub fn pane(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let mut navigation_action: Option<BattleEditorAction> = None;
        let mut changed = false;

        let is_left = world.resource::<BattleEditorState>().is_left_team;

        // Team selector header
        ui.horizontal(|ui| {
            ui.label("Editing:");
            if ui.selectable_label(is_left, "Left Team").clicked() {
                world.resource_mut::<BattleEditorState>().is_left_team = true;

                let battle_data = world.remove_resource::<BattleData>().unwrap();
                let team_id = battle_data
                    .teams_world
                    .as_context()
                    .id(battle_data.team_left)
                    .unwrap_or_default();
                world.insert_resource(battle_data);

                navigation_action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::Team(
                    team_id,
                )));
            }
            if ui.selectable_label(!is_left, "Right Team").clicked() {
                world.resource_mut::<BattleEditorState>().is_left_team = false;

                let battle_data = world.remove_resource::<BattleData>().unwrap();
                let team_id = battle_data
                    .teams_world
                    .as_context()
                    .id(battle_data.team_right)
                    .unwrap_or_default();
                world.insert_resource(battle_data);

                navigation_action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::Team(
                    team_id,
                )));
            }
        });

        ui.separator();

        // Navigation breadcrumb
        let state = world.resource::<BattleEditorState>();
        ui.horizontal(|ui| {
            if !state.navigation_stack.is_empty() {
                if ui.button("← Back").clicked() {
                    navigation_action = Some(BattleEditorAction::GoBack);
                }
                ui.separator();
            }

            // Show current node
            if let Some(current) = &state.current_node {
                match current {
                    BattleEditorNode::Team(id) => {
                        format!("[tw Team #{}]", id).cstr().label(ui);
                    }
                    BattleEditorNode::House(id) => {
                        format!("[tw House #{}]", id).cstr().label(ui);
                    }
                    BattleEditorNode::Unit(id) => {
                        format!("[tw Unit #{}]", id).cstr().label(ui);
                    }
                }
            } else {
                ui.label("Select a component to edit");
            }
        });

        // Main content area - NO SCROLL AREA
        let current_node = world.resource::<BattleEditorState>().current_node.clone();
        let result = if let Some(current) = &current_node {
            match current {
                BattleEditorNode::Team(id) => Self::render_team_editor(*id, ui, world),
                BattleEditorNode::House(id) => Self::render_house_editor(*id, ui, world),
                BattleEditorNode::Unit(id) => Self::render_unit_editor(*id, ui, world),
            }
        } else {
            Self::render_team_selector(ui, world)
        };

        match result {
            Ok((Some(action), _)) => navigation_action = Some(action),
            Ok((_, true)) => changed = true,
            Err(err) => {
                err.ui(ui);
            }
            _ => {}
        }

        // Handle navigation actions
        if let Some(action) = navigation_action {
            let mut state = world.resource_mut::<BattleEditorState>();
            match action {
                BattleEditorAction::GoBack => {
                    if let Some(prev_node) = state.navigation_stack.pop() {
                        state.current_node = Some(prev_node);
                    }
                }
                BattleEditorAction::Navigate(node) => {
                    if let Some(current) = state.current_node.take() {
                        state.navigation_stack.push(current);
                    }
                    state.current_node = Some(node);
                }
                BattleEditorAction::SetCurrent(node) => {
                    state.current_node = Some(node);
                    state.navigation_stack.clear();
                }
            }
        }

        if changed {
            Self::save_team_changes(world);
        }

        Ok(())
    }

    fn render_team_selector(
        ui: &mut Ui,
        world: &mut World,
    ) -> NodeResult<(Option<BattleEditorAction>, bool)> {
        let mut action = None;
        let is_left = world.resource::<BattleEditorState>().is_left_team;

        ui.heading(if is_left { "Left Team" } else { "Right Team" });
        ui.separator();

        let battle_data = world.resource::<BattleData>();
        let result = battle_data.teams_world.with_context(|context| {
            let team_entity = if is_left {
                battle_data.team_left
            } else {
                battle_data.team_right
            };

            if let Ok(team) = context.load_entity::<NTeam>(team_entity) {
                // Use render system for team display
                team.title(context).label(ui);
                ui.add_space(8.0);

                if ui.button("Edit Team").clicked() {
                    action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::Team(
                        context.id(team_entity)?,
                    )));
                }
            }
            Ok(())
        });

        result?;
        Ok((action, false))
    }

    fn render_team_editor(
        team_id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> NodeResult<(Option<BattleEditorAction>, bool)> {
        let mut action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = battle_data.teams_world.with_context_mut(|context| {
            const ACTION_DEFAULT_UNIT: &str = "Add Default Unit";
            const ACTION_UNIT_EDITOR: &str = "Open Unit Editor";
            const ACTION_DELETE_UNIT: &str = "Delete Unit";

            let team_entity = context.entity(team_id)?;

            // Use TeamEditor widget for the main team editing
            let team_editor = TeamEditor::new(team_entity)
                .empty_slot_action(ACTION_DEFAULT_UNIT)
                .filled_slot_action(ACTION_UNIT_EDITOR)
                .filled_slot_action(ACTION_DELETE_UNIT);
            let actions = team_editor.ui(ui, context)?;

            // Process team editor actions
            for team_action in actions {
                match team_action {
                    TeamAction::MoveUnit { unit_id, target } => {
                        Self::handle_move_unit(unit_id, target, context)?;
                        changed = true;
                    }
                    TeamAction::ContextMenuAction {
                        slot_id,
                        action_name,
                        unit_id,
                    } => {
                        if action_name == ACTION_DEFAULT_UNIT {
                            Self::handle_add_default_unit_to_slot(team_entity, slot_id, context)?;
                            changed = true;
                        } else if action_name == ACTION_UNIT_EDITOR {
                            if let Some(unit_id) = unit_id {
                                action = Some(BattleEditorAction::Navigate(
                                    BattleEditorNode::Unit(unit_id),
                                ));
                            }
                        } else if action_name == ACTION_DELETE_UNIT {
                            if let Some(unit_id) = unit_id {
                                Self::handle_delete_unit(unit_id, context)?;
                                changed = true;
                            }
                        }
                    }
                    TeamAction::AddSlot { fusion_id } => {
                        Self::handle_add_slot(fusion_id, context)?;
                        changed = true;
                    }
                    TeamAction::ChangeActionRange {
                        slot_id,
                        start,
                        length,
                    } => {
                        Self::handle_change_action_range(slot_id, start, length, context)?;
                        changed = true;
                    }
                    TeamAction::BenchUnit { unit_id } => {
                        Self::handle_bench_unit(unit_id, context)?;
                        changed = true;
                    }
                    TeamAction::ChangeTrigger { fusion_id, trigger } => {
                        Self::handle_change_trigger(fusion_id, trigger, context)?;
                        changed = true;
                    }
                }
            }

            ui.separator();
            "[h2 Houses]".cstr().label(ui);

            // Use generic collection management for houses
            let houses: Vec<NHouse> = context
                .collect_children::<NHouse>(team_id)?
                .into_iter()
                .cloned()
                .collect();
            let (houses_changed, houses_action) = Self::manage_parts::<NHouse>(
                ui,
                context,
                team_entity,
                "House",
                || Ok(houses.clone()),
                |house| {
                    Some(BattleEditorAction::Navigate(BattleEditorNode::House(
                        house.id(),
                    )))
                },
            )?;
            changed |= houses_changed;
            if action.is_none() {
                action = houses_action;
            }

            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn render_house_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> NodeResult<(Option<BattleEditorAction>, bool)> {
        let mut action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = battle_data.teams_world.with_context_mut(|ctx| {
            let entity = ctx.entity(id)?;
            let mut house = ctx.load_entity::<NHouse>(entity).cloned()?;
            let units = house.units_load(ctx)?.clone();
            ui.group(|ui| {
                house.title(ctx).label(ui);
                if house.edit(ctx, ui).changed() {
                    house.spawn(ctx, Some(entity)).log();
                    changed = true;
                }
            });
            changed |= Self::manage_part::<NHouseColor>(ui, ctx, id)?.1;
            ui.collapsing("Ability", |ui| -> NodeResult<()> {
                let (ability_id, part_changed) = Self::manage_part::<NAbilityMagic>(ui, ctx, id)?;
                changed |= part_changed;
                if ability_id > 0 {
                    let (desc_id, part_changed) =
                        Self::manage_part::<NAbilityDescription>(ui, ctx, ability_id)?;
                    changed |= part_changed;
                    if desc_id > 0 {
                        Self::manage_part::<NAbilityEffect>(ui, ctx, desc_id)?;
                    }
                }
                Ok(())
            })
            .body_returned
            .unwrap_or(Ok(()))?;

            changed |= Self::manage_part::<NStatusMagic>(ui, ctx, id)?.1;

            ui.separator();
            "[h2 Units]".cstr().label(ui);

            // Use generic collection management for units
            let (units_changed, units_action) = Self::manage_parts::<NUnit>(
                ui,
                ctx,
                entity,
                "Unit",
                || Ok(units.clone()),
                |unit| {
                    Some(BattleEditorAction::Navigate(BattleEditorNode::Unit(
                        unit.id(),
                    )))
                },
            )?;
            changed |= units_changed;
            if action.is_none() {
                action = units_action;
            }

            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn render_unit_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> NodeResult<(Option<BattleEditorAction>, bool)> {
        let action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = battle_data.teams_world.with_context_mut(|ctx| {
            let entity = ctx.entity(id)?;
            let mut unit = ctx.load_entity::<NUnit>(entity).cloned()?;

            ui.group(|ui| {
                unit.title(ctx).label(ui);
                if unit.edit(ctx, ui).changed() {
                    unit.clone().spawn(ctx, Some(entity)).log();
                    changed = true;
                }
            });

            ui.separator();

            changed |= Self::manage_part::<NUnitStats>(ui, ctx, id)?.1;
            changed |= Self::manage_part::<NUnitState>(ui, ctx, id)?.1;

            ui.collapsing("Description", |ui| -> NodeResult<()> {
                let (desc_id, part_changed) = Self::manage_part::<NUnitDescription>(ui, ctx, id)?;
                changed |= part_changed;
                if desc_id > 0 {
                    changed |= Self::manage_part::<NUnitRepresentation>(ui, ctx, desc_id)?.1;
                    changed |= Self::manage_part::<NUnitBehavior>(ui, ctx, desc_id)?.1;
                }
                Ok(())
            })
            .body_returned
            .unwrap_or(Ok(()))?;

            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn save_team_changes(world: &mut World) {
        world.resource_mut::<ReloadData>().reload_requested = true;
    }

    fn handle_move_unit(unit_id: u64, target_id: u64, ctx: &mut ClientContext) -> NodeResult<()> {
        // Find and clear the old slot that references this unit
        if let Ok(old_slot_id) = ctx.first_parent(unit_id, NodeKind::NFusionSlot) {
            ctx.remove_link(old_slot_id, unit_id)?;
        }

        // Set the unit reference in the target slot
        ctx.load::<NFusionSlot>(target_id)?;
        ctx.add_link(target_id, unit_id)?;
        Ok(())
    }

    fn handle_bench_unit(unit_id: u64, ctx: &mut ClientContext) -> NodeResult<()> {
        let slot_id = ctx.first_parent(unit_id, NodeKind::NFusionSlot)?;
        ctx.remove_link(slot_id, unit_id)?;
        Ok(())
    }

    fn handle_add_default_unit_to_slot(
        team_entity: Entity,
        slot_id: u64,
        ctx: &mut ClientContext,
    ) -> NodeResult<()> {
        let slot_entity = ctx.entity(slot_id)?;

        let house_entity = {
            let existing_houses = ctx.collect_children::<NHouse>(ctx.id(team_entity)?)?;
            let default_house = existing_houses
                .iter()
                .find(|h| h.house_name == "Default House");

            if let Some(existing_house) = default_house {
                existing_house.entity(ctx)?
            } else {
                let house = NHouse::placeholder(next_id());
                let house_entity = ctx.world_mut()?.spawn_empty().id();
                house.spawn(ctx, Some(house_entity))?;
                ctx.add_link_entities(team_entity, house_entity)?;
                house_entity
            }
        };

        let unit = NUnit::placeholder(next_id());
        let unit_entity = ctx.world_mut()?.spawn_empty().id();
        let unit_id = unit.id();
        unit.spawn(ctx, Some(unit_entity))?;
        ctx.add_link_entities(house_entity, unit_entity)?;

        // Update the slot's unit reference and respawn it to persist the change
        let mut slot = ctx.load_entity::<NFusionSlot>(slot_entity)?.clone();
        slot.unit = Ref::new_id(unit_id);
        slot.spawn(ctx, Some(slot_entity))?;

        Ok(())
    }

    fn handle_add_slot(fusion_id: u64, context: &mut ClientContext) -> NodeResult<()> {
        let fusion_entity = context.entity(fusion_id)?;
        let existing_slots = context.collect_children::<NFusionSlot>(fusion_id)?;
        let next_index = existing_slots.len() as i32;

        let slot_id = next_id();
        let new_slot = NFusionSlot::new(slot_id, next_index, default());
        let slot_entity = context.world_mut()?.spawn_empty().id();
        new_slot.spawn(context, Some(slot_entity))?;
        context.add_link_entities(fusion_entity, slot_entity)?;

        Ok(())
    }

    fn handle_delete_unit(unit_id: u64, ctx: &mut ClientContext) -> NodeResult<()> {
        ctx.despawn(unit_id).log();
        Ok(())
    }

    fn handle_change_action_range(
        slot_id: u64,
        start: u8,
        length: u8,
        ctx: &mut ClientContext,
    ) -> NodeResult<()> {
        let slot_entity = ctx.entity(slot_id)?;
        let mut slot = ctx.load_entity::<NFusionSlot>(slot_entity)?.clone();
        slot.actions.start = start;
        slot.actions.length = length;
        slot.spawn(ctx, Some(slot_entity))?;
        Ok(())
    }

    fn handle_change_trigger(
        fusion_id: u64,
        trigger: u64,
        ctx: &mut ClientContext,
    ) -> NodeResult<()> {
        let fusion_entity = ctx.entity(fusion_id)?;
        let mut fusion = ctx.load_entity::<NFusion>(fusion_entity)?.clone();
        fusion.trigger_unit = trigger;
        fusion.spawn(ctx, Some(fusion_entity))?;
        Ok(())
    }
}

enum BattleEditorAction {
    GoBack,
    Navigate(BattleEditorNode),
    SetCurrent(BattleEditorNode),
}
