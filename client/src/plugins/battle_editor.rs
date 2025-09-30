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
    fn manage_part<T: FPlaceholder + FEdit + Node + Clone + 'static>(
        ui: &mut Ui,
        context: &mut ClientContext,
        id: u64,
        owner: u64,
        is_child_part: bool, // true if T is a child of part_owner_entity
    ) -> Result<(u64, bool), ExpressionError> {
        let mut changed = false;
        let mut part_id = 0;
        let entity = context.entity(id)?;
        let part_result = if is_child_part {
            context.first_child::<T>(id)
        } else {
            context.first_parent::<T>(id)
        };

        match part_result {
            Ok(part) => {
                part_id = part.id();
                let part_entity = part.entity();
                let mut part = part.clone();

                ui.group(|ui| {
                    ui.label(format!("{}:", part.kind()));
                    if part.edit(context, ui).changed() {
                        part.clone().unpack_entity(context, part_entity).log();
                        changed = true;
                    }
                    if "[red Delete]".cstr().button(ui).clicked() {
                        context.despawn(part_entity).log();
                        changed = true;
                    }
                });
            }
            Err(_) => {
                if ui.button(format!("➕ Add {}", T::kind_s())).clicked() {
                    let part = T::placeholder(owner);
                    part_id = part.id();
                    let part_entity = context.world_mut()?.spawn_empty().id();
                    part.unpack_entity(context, part_entity)?;

                    if is_child_part {
                        context.link_parent_child_entity(entity, part_entity)?;
                    } else {
                        context.link_parent_child_entity(part_entity, entity)?;
                    }
                    changed = true;
                }
            }
        }

        Ok((part_id, changed))
    }

    // Generic helper for managing collections of child nodes
    fn manage_parts<T: FPlaceholder + FTitle + FCopy + FPaste + Node + Clone + 'static>(
        ui: &mut Ui,
        context: &mut ClientContext,
        parent_entity: Entity,
        owner: u64,
        collection_name: &str,
        get_children_fn: impl Fn() -> Result<Vec<T>, ExpressionError>,
        on_edit: impl Fn(T) -> Option<BattleEditorAction>,
    ) -> Result<(bool, Option<BattleEditorAction>), ExpressionError> {
        let mut changed = false;
        let mut navigation_action = None;

        ui.label(format!("{}:", collection_name));
        let children = get_children_fn()?;
        let mut children_to_delete = Vec::new();
        let mut children_to_replace = Vec::new();

        for child in children {
            ui.horizontal(|ui| {
                child.title(context).label(ui);

                if ui.button("Edit").clicked() {
                    navigation_action = on_edit(child.clone());
                }

                if ui.button("📋 Copy").clicked() {
                    child.copy_to_clipboard();
                }

                if ui.button("📋 Paste").clicked() {
                    if let Some(replacement) = T::paste_from_clipboard() {
                        children_to_replace.push((child.entity(), replacement));
                    }
                }

                if ui.button("🗑 Delete").clicked() {
                    children_to_delete.push(child.entity());
                }
            });
        }

        for (entity, replacement) in children_to_replace {
            if let Err(e) = replacement.unpack_entity(context, entity) {
                error!("Failed to replace {}: {}", collection_name, e);
            } else {
                changed = true;
            }
        }

        for entity in children_to_delete {
            context.despawn(entity).log();
            changed = true;
        }

        if ui.button(format!("➕ Add {}", collection_name)).clicked() {
            let item = T::placeholder(owner);
            let item_entity = context.world_mut()?.spawn_empty().id();
            item.unpack_entity(context, item_entity)?;
            context.link_parent_child_entity(parent_entity, item_entity)?;
            changed = true;
        }

        Ok((changed, navigation_action))
    }

    pub fn pane(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut navigation_action: Option<BattleEditorAction> = None;
        let mut changed = false;

        let is_left = world.resource::<BattleEditorState>().is_left_team;

        // Team selector header
        ui.horizontal(|ui| {
            ui.label("Editing:");
            if ui.selectable_label(is_left, "Left Team").clicked() {
                world.resource_mut::<BattleEditorState>().is_left_team = true;

                let mut battle_data = world.remove_resource::<BattleData>().unwrap();
                let team_id = Context::from_world_r(&mut battle_data.teams_world, |context| {
                    context.id(battle_data.team_left)
                })
                .unwrap_or(0);
                world.insert_resource(battle_data);

                navigation_action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::Team(
                    team_id,
                )));
            }
            if ui.selectable_label(!is_left, "Right Team").clicked() {
                world.resource_mut::<BattleEditorState>().is_left_team = false;

                let mut battle_data = world.remove_resource::<BattleData>().unwrap();
                let team_id = Context::from_world_r(&mut battle_data.teams_world, |context| {
                    context.id(battle_data.team_right)
                })
                .unwrap_or(0);
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
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let is_left = world.resource::<BattleEditorState>().is_left_team;

        ui.heading(if is_left { "Left Team" } else { "Right Team" });
        ui.separator();

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = Context::from_world_r(&mut battle_data.teams_world, |context| {
            let team_entity = if is_left {
                battle_data.team_left
            } else {
                battle_data.team_right
            };

            if let Ok(team) = context.component::<NTeam>(team_entity) {
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

        world.insert_resource(battle_data);
        result?;
        Ok((action, false))
    }

    fn render_team_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = Context::from_world_r(&mut battle_data.teams_world, |context| {
            const ACTION_DEFAULT_UNIT: &str = "Add Default Unit";
            const ACTION_UNIT_EDITOR: &str = "Open Unit Editor";
            const ACTION_DELETE_UNIT: &str = "Delete Unit";

            let team_entity = context.entity(id)?;

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
            let team = context.component::<NTeam>(team_entity)?;
            let team_owner = team.owner;
            let houses: Vec<NHouse> = context
                .collect_children_components::<NHouse>(id)?
                .into_iter()
                .cloned()
                .collect();
            let (houses_changed, houses_action) = Self::manage_parts::<NHouse>(
                ui,
                context,
                team_entity,
                team_owner,
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
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = Context::from_world_r(&mut battle_data.teams_world, |context| {
            let entity = context.entity(id)?;
            let mut house = context.component::<NHouse>(entity).cloned()?;
            let owner = house.owner;
            let mut units = vec![]; // TODO: Uncomment after code generation
            if let Ok(ability) = house.ability_load(context) {
                units.extend(ability.units_load(context).into_iter().cloned());
            }
            if let Ok(status) = house.status_load(context) {
                units.extend(status.units_load(context).into_iter().cloned());
            }
            ui.group(|ui| {
                house.title(context).label(ui);
                if house.edit(context, ui).changed() {
                    house.unpack_entity(context, entity).log();
                    changed = true;
                }
            });
            changed |= Self::manage_part::<NHouseColor>(ui, context, id, owner, false)?.1;
            ui.collapsing("Ability", |ui| -> Result<(), ExpressionError> {
                let (id, part_changed) =
                    Self::manage_part::<NAbilityMagic>(ui, context, id, owner, false)?;
                changed |= part_changed;
                if id > 0 {
                    let (id, part_changed) =
                        Self::manage_part::<NAbilityDescription>(ui, context, id, owner, false)?;
                    changed |= part_changed;
                    if id > 0 {
                        Self::manage_part::<NAbilityEffect>(ui, context, id, owner, false)?;
                    }
                }
                Ok(())
            })
            .body_returned
            .unwrap_or(Ok(()))?;

            changed |= Self::manage_part::<NStatusMagic>(ui, context, id, owner, false)?.1;

            ui.separator();
            "[h2 Units]".cstr().label(ui);

            // Use generic collection management for units
            let (units_changed, units_action) = Self::manage_parts::<NUnit>(
                ui,
                context,
                entity,
                owner,
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
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = Context::from_world_r(&mut battle_data.teams_world, |context| {
            let entity = context.entity(id)?;
            let mut unit = context.component::<NUnit>(entity).cloned()?;

            ui.group(|ui| {
                unit.title(context).label(ui);
                if unit.edit(context, ui).changed() {
                    unit.clone().unpack_entity(context, entity).log();
                    changed = true;
                }
            });

            ui.separator();

            changed |= Self::manage_part::<NUnitStats>(ui, context, unit.id, unit.owner, true)?.1;

            changed |= Self::manage_part::<NUnitState>(ui, context, unit.id, unit.owner, true)?.1;
            changed |=
                Self::manage_part::<NUnitDescription>(ui, context, unit.id, unit.owner, true)?.1;

            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn save_team_changes(world: &mut World) {
        world.resource_mut::<ReloadData>().reload_requested = true;
    }

    fn handle_move_unit(
        unit_id: u64,
        target_id: u64,
        context: &mut ClientContext,
    ) -> Result<(), ExpressionError> {
        if let Ok(slot) = context.first_child::<NFusionSlot>(unit_id) {
            context.unlink_parent_child(unit_id, slot.id)?;
        }
        context.link_parent_child(unit_id, target_id)?;
        Ok(())
    }

    fn handle_bench_unit(unit_id: u64, context: &mut ClientContext) -> Result<(), ExpressionError> {
        if let Ok(slot) = context.first_child::<NFusionSlot>(unit_id) {
            context.unlink_parent_child(unit_id, slot.id)?;
        } else {
            return Err(
                ExpressionErrorVariants::Custom("Unit is not in a slot".to_string()).into(),
            );
        };
        Ok(())
    }

    fn handle_add_default_unit_to_slot(
        team_entity: Entity,
        slot_id: u64,
        context: &mut ClientContext,
    ) -> Result<(), ExpressionError> {
        let slot_entity = context.entity(slot_id)?;
        let team = context.component::<NTeam>(team_entity)?;
        let team_owner = team.owner;

        let house_entity = {
            let existing_houses =
                context.collect_children_components::<NHouse>(context.id(team_entity)?)?;
            let default_house = existing_houses
                .iter()
                .find(|h| h.house_name == "Default House");

            if let Some(existing_house) = default_house {
                existing_house.entity()
            } else {
                let house = NHouse::placeholder(team_owner);
                let house_entity = context.world_mut()?.spawn_empty().id();
                house.clone().unpack_entity(context, house_entity)?;
                context.link_parent_child_entity(team_entity, house_entity)?;
                house_entity
            }
        };

        let unit = NUnit::placeholder(team_owner);
        let unit_entity = context.world_mut()?.spawn_empty().id();
        let unit_id = unit.id();
        unit.unpack_entity(context, unit_entity)?;
        context.link_parent_child_entity(house_entity, unit_entity)?;
        let unit_entity = context.entity(unit_id)?;
        context.link_parent_child_entity(unit_entity, slot_entity)?;

        Ok(())
    }

    fn handle_add_slot(fusion_id: u64, context: &mut ClientContext) -> Result<(), ExpressionError> {
        let fusion_entity = context.entity(fusion_id)?;
        let fusion = context.component::<NFusion>(fusion_entity)?;

        // Find the next slot index
        let existing_slots = context.collect_parents_components::<NFusionSlot>(fusion_id)?;
        let next_index = existing_slots.len() as i32;

        // Create new fusion slot
        let new_slot = NFusionSlot::new(fusion.owner, next_index, default());
        let slot_entity = context.world_mut()?.spawn_empty().id();
        new_slot.unpack_entity(context, slot_entity)?;
        context.link_parent_child_entity(slot_entity, fusion_entity)?;

        Ok(())
    }

    fn handle_delete_unit(
        unit_id: u64,
        context: &mut ClientContext,
    ) -> Result<(), ExpressionError> {
        let unit_entity = context.entity(unit_id)?;
        context.despawn(unit_entity).log();
        Ok(())
    }

    fn handle_change_action_range(
        slot_id: u64,
        start: u8,
        length: u8,
        context: &mut ClientContext,
    ) -> Result<(), ExpressionError> {
        let mut slot = context.component_mut::<NFusionSlot>(context.entity(slot_id)?)?;
        slot.actions.start = start;
        slot.actions.length = length;
        Ok(())
    }

    fn handle_change_trigger(
        fusion_id: u64,
        trigger: u64,
        context: &mut ClientContext,
    ) -> Result<(), ExpressionError> {
        let mut fusion = context.component_mut::<NFusion>(context.entity(fusion_id)?)?;
        fusion.trigger_unit = trigger;
        Ok(())
    }
}

enum BattleEditorAction {
    GoBack,
    Navigate(BattleEditorNode),
    SetCurrent(BattleEditorNode),
}
