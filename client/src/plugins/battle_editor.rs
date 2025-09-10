use super::*;
use crate::nodes::*;
use crate::resources::*;
use crate::ui::render::*;
use crate::ui::*;
use crate::utils::*;

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
                err.cstr().notify_error(world);
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
                team.render(context).tag(ui);
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

            // Render houses using the new render system
            let houses = context.collect_children_components::<NHouse>(id)?;
            let mut houses_to_delete = Vec::new();
            let mut houses_to_replace = Vec::new();

            for house in houses {
                ui.horizontal(|ui| {
                    // Use render system with context menu
                    let response = house
                        .render(context)
                        .with_menu()
                        .add_copy()
                        .add_paste()
                        .add_delete()
                        .title(ui);

                    if ui.button("Edit").clicked() {
                        action = Some(BattleEditorAction::Navigate(BattleEditorNode::House(
                            house.id(),
                        )));
                    }

                    if let Some(_) = response.deleted() {
                        houses_to_delete.push(house.entity());
                    }

                    if let Some(replacement) = response.replaced() {
                        houses_to_replace.push((house.entity(), replacement.clone()));
                    }
                });
            }

            for (entity, replacement) in houses_to_replace {
                if let Err(e) = replacement.unpack_entity(context, entity) {
                    error!("Failed to replace house: {}", e);
                } else {
                    changed = true;
                }
            }

            for entity in houses_to_delete {
                context.despawn(entity).log();
                changed = true;
            }

            if ui
                .button(format!("➕ Add {}", NHouse::kind_s().cstr()))
                .clicked()
            {
                Self::create_node::<NHouse>(context, id).add_parent(context.world_mut()?, id);
                changed = true;
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
            let house = context.component::<NHouse>(entity).cloned()?;

            // Basic house info and editing
            ui.group(|ui| {
                house.render(context).info().label(ui);

                let mut house_mut = house.clone();
                if house_mut.render_mut(context).edit(ui) {
                    house_mut.unpack_entity(context, entity).log();
                    changed = true;
                }
            });

            ui.separator();

            // House color
            if let Ok(color) = context.first_parent::<NHouseColor>(id) {
                let color_entity = color.entity();
                let color_clone = color.clone();
                ui.group(|ui| {
                    color_clone.render(context).title_label(ui);
                    let mut color_mut = color_clone.clone();
                    if color_mut.render_mut(context).edit(ui) {
                        color_mut.unpack_entity(context, color_entity).log();
                        changed = true;
                    }
                });
            }

            // Abilities - directly edit the effect actions list
            if let Ok(mut ability) = house.ability_load(context).cloned() {
                ui.separator();
                ui.collapsing("Ability", |ui| {
                    ui.group(|ui| {
                        // Basic ability info
                        ability.render(context).info().label(ui);
                        if ability.render_mut(context).edit(ui) {
                            ability
                                .clone()
                                .unpack_entity(context, ability.entity())
                                .log();
                            changed = true;
                        }

                        // Effect actions list editor
                        if let Ok(mut effect) = ability
                            .description_load(context)
                            .and_then(|d| d.effect_load(context))
                            .cloned()
                        {
                            let effect_entity = effect.entity();
                            ui.separator();
                            ui.label("Effect Actions:");

                            if effect.actions.render_mut(context).edit_recursive_list(ui) {
                                effect.unpack_entity(context, effect_entity).log();
                                changed = true;
                            }
                        }
                    });
                });
            }

            // Statuses
            if let Ok(mut status) = house.status_load(context).cloned() {
                ui.separator();
                ui.collapsing("Status", |ui| {
                    ui.group(|ui| {
                        // Basic status info
                        status.render(context).info().label(ui);
                        if status.render_mut(context).edit(ui) {
                            status.clone().unpack_entity(context, status.entity()).log();
                            changed = true;
                        }

                        // Behavior reactions list editor
                        if let Ok(mut behavior) = status
                            .description_load(context)
                            .and_then(|d| d.behavior_load(context))
                            .cloned()
                        {
                            let behavior_entity = behavior.entity();
                            ui.separator();
                            ui.label("Behavior Reactions:");

                            if behavior
                                .reactions
                                .render_mut(context)
                                .edit_recursive_list(ui)
                            {
                                behavior.unpack_entity(context, behavior_entity).log();
                                changed = true;
                            }
                        }

                        // Representation material editor
                        if let Ok(repr) = status.representation_load(context).cloned() {
                            let repr_entity = repr.entity();
                            let mut repr_mut = repr.clone();
                            ui.separator();
                            ui.label("Representation:");

                            if repr_mut.material.render_mut(context).edit(ui) {
                                repr_mut.unpack_entity(context, repr_entity).log();
                                changed = true;
                            }
                        }
                    });
                });
            }

            ui.separator();
            "[h2 Units]".cstr().label(ui);

            // Render units
            let units = context.collect_children_components::<NUnit>(id)?;
            let mut units_to_delete = Vec::new();
            let mut units_to_replace = Vec::new();

            for unit in units {
                ui.horizontal(|ui| {
                    let response = unit
                        .render(context)
                        .with_menu()
                        .add_copy()
                        .add_paste()
                        .add_delete()
                        .title(ui);

                    if ui.button("Edit").clicked() {
                        action = Some(BattleEditorAction::Navigate(BattleEditorNode::Unit(
                            unit.id(),
                        )));
                    }

                    if let Some(_) = response.deleted() {
                        units_to_delete.push(unit.entity());
                    }

                    if let Some(replacement) = response.replaced() {
                        units_to_replace.push((unit.entity(), replacement.clone()));
                    }
                });
            }

            for (entity, replacement) in units_to_replace {
                if let Err(e) = replacement.unpack_entity(context, entity) {
                    error!("Failed to replace unit: {}", e);
                } else {
                    changed = true;
                }
            }

            for entity in units_to_delete {
                context.despawn(entity).log();
                changed = true;
            }

            if ui.button(format!("➕ Add Unit")).clicked() {
                Self::create_default_unit(context, entity, id)?;
                changed = true;
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
            let unit = context.component::<NUnit>(entity).cloned()?;

            // Basic unit info
            ui.group(|ui| {
                unit.render(context).info().label(ui);

                let mut unit_mut = unit.clone();
                if unit_mut.render_mut(context).edit(ui) {
                    unit_mut.unpack_entity(context, entity).log();
                    changed = true;
                }
            });

            ui.separator();

            // Stats
            if let Ok(stats) = unit.stats_load(context) {
                let stats_entity = stats.entity();
                let stats_clone = stats.clone();
                ui.group(|ui| {
                    ui.label("Stats:");
                    stats_clone.render(context).title_label(ui);

                    let mut stats_mut = stats_clone.clone();
                    if stats_mut.render_mut(context).edit(ui) {
                        stats_mut.unpack_entity(context, stats_entity).log();
                        changed = true;
                    }
                });
            }

            // State
            if let Ok(state) = unit.state_load(context) {
                let state_entity = state.entity();
                let state_clone = state.clone();
                ui.separator();
                ui.group(|ui| {
                    ui.label("State:");
                    state_clone.render(context).title_label(ui);

                    let mut state_mut = state_clone.clone();
                    if state_mut.render_mut(context).edit(ui) {
                        state_mut.unpack_entity(context, state_entity).log();
                        changed = true;
                    }
                });
            }

            // Description
            if let Ok(desc) = unit.description_load(context).cloned() {
                let desc_entity = desc.entity();

                ui.separator();
                ui.group(|ui| {
                    ui.label("Description:");
                    desc.render(context).title_label(ui);

                    let mut desc_mut = desc.clone();
                    if desc_mut.render_mut(context).edit(ui) {
                        desc_mut.unpack_entity(context, desc_entity).log();
                        changed = true;
                    }
                });

                // Behavior
                let behavior_opt = desc.behavior_load(context).ok().cloned();
                if let Some(behavior) = behavior_opt {
                    let behavior_entity = behavior.entity();
                    let mut behavior_mut = behavior.clone();

                    ui.separator();
                    ui.collapsing("Behavior", |ui| {
                        ui.group(|ui| {
                            // Magic type
                            let mut magic_type_changed = false;
                            ui.horizontal(|ui| {
                                ui.label("Magic Type:");
                                if behavior_mut.magic_type.show_mut(context, ui) {
                                    magic_type_changed = true;
                                }
                            });

                            ui.separator();

                            // Edit the reaction directly
                            ui.label("Reaction:");
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Trigger:");
                                    if behavior_mut.reaction.trigger.show_mut(context, ui) {
                                        magic_type_changed = true;
                                    }
                                });

                                ui.label("Actions:");
                                if behavior_mut
                                    .reaction
                                    .actions
                                    .render_mut(context)
                                    .edit_recursive_list(ui)
                                {
                                    magic_type_changed = true;
                                }
                            });

                            if magic_type_changed {
                                behavior_mut.unpack_entity(context, behavior_entity).log();
                                changed = true;
                            }
                        });
                    });
                }

                // Representation
                if let Ok(repr) = desc.representation_load(context).cloned() {
                    let repr_entity = repr.entity();
                    let mut repr_mut = repr.clone();

                    ui.separator();
                    ui.collapsing("Representation (Material)", |ui| {
                        if repr_mut
                            .material
                            .render_mut(context)
                            .recursive_edit_tree(ui)
                        {
                            repr_mut.unpack_entity(context, repr_entity).log();
                            changed = true;
                        }
                    });
                }
            }

            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn create_node<T>(context: &mut Context, owner: u64) -> u64
    where
        T: Node + 'static,
    {
        let mut component = T::default();
        component.set_owner(owner);
        let id = next_id();
        component.set_id(id);

        // Set default values for specific types
        match T::kind_s().cstr().as_str() {
            "NHouse" => {
                if let Some(house) =
                    (&mut component as &mut dyn std::any::Any).downcast_mut::<NHouse>()
                {
                    house.house_name = "New House".to_string();
                }
            }
            "NFusion" => {
                if let Some(fusion) =
                    (&mut component as &mut dyn std::any::Any).downcast_mut::<NFusion>()
                {
                    fusion.pwr = 1;
                    fusion.hp = 1;
                    fusion.dmg = 1;
                    fusion.actions_limit = 1;
                }
            }
            _ => {}
        }

        let component_entity = context.world_mut().unwrap().spawn_empty().id();
        component.unpack_entity(context, component_entity).log();
        id
    }

    fn create_default_unit(
        context: &mut Context,
        house_entity: Entity,
        owner: u64,
    ) -> Result<u64, ExpressionError> {
        let unit_stats = NUnitStats::new_full(owner, 1, 1);
        let unit_state = NUnitState::new_full(owner, 1);
        let unit_behavior = NUnitBehavior::new_full(
            owner,
            Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::noop],
            },
            MagicType::Ability,
        );
        let unit_representation = NUnitRepresentation::new_full(
            owner,
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        );

        let unit_description = NUnitDescription::new_full(
            owner,
            "Default Description".into(),
            MagicType::Ability,
            Trigger::BattleStart,
            unit_representation,
            unit_behavior,
        );

        let unit = NUnit::new_full(
            owner,
            "New Unit".to_string(),
            unit_description,
            unit_stats,
            unit_state,
        );

        let unit_entity = context.world_mut()?.spawn_empty().id();
        let unit_id = unit.id();
        unit.unpack_entity(context, unit_entity)?;
        context.link_parent_child_entity(house_entity, unit_entity)?;

        Ok(unit_id)
    }

    fn save_team_changes(world: &mut World) {
        world.resource_mut::<ReloadData>().reload_requested = true;
    }

    fn handle_move_unit(
        unit_id: u64,
        target_id: u64,
        context: &mut Context,
    ) -> Result<(), ExpressionError> {
        if let Ok(slot) = context.first_child::<NFusionSlot>(unit_id) {
            context.unlink_parent_child(unit_id, slot.id)?;
        }
        context.link_parent_child(unit_id, target_id)?;
        Ok(())
    }

    fn handle_bench_unit(unit_id: u64, context: &mut Context) -> Result<(), ExpressionError> {
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
        context: &mut Context,
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
                let house_color = NHouseColor::new_full(team_owner, default());
                let action_ability =
                    NAbilityMagic::new_full(team_owner, "Default Action".to_string(), default());
                let status_ability = NStatusMagic::new_full(
                    team_owner,
                    "Default Status".to_string(),
                    default(),
                    default(),
                );
                let house = NHouse::new_full(
                    team_owner,
                    "Default House".to_string(),
                    house_color,
                    action_ability,
                    status_ability,
                    vec![],
                );

                let house_entity = context.world_mut()?.spawn_empty().id();
                house.clone().unpack_entity(context, house_entity)?;
                context.link_parent_child_entity(team_entity, house_entity)?;
                house_entity
            }
        };

        let unit_id = Self::create_default_unit(context, house_entity, team_owner)?;
        let unit_entity = context.entity(unit_id)?;
        context.link_parent_child_entity(unit_entity, slot_entity)?;

        Ok(())
    }

    fn handle_add_slot(fusion_id: u64, context: &mut Context) -> Result<(), ExpressionError> {
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

    fn handle_delete_unit(unit_id: u64, context: &mut Context) -> Result<(), ExpressionError> {
        let unit_entity = context.entity(unit_id)?;
        context.despawn(unit_entity).log();
        Ok(())
    }

    fn handle_change_action_range(
        slot_id: u64,
        start: u8,
        length: u8,
        context: &mut Context,
    ) -> Result<(), ExpressionError> {
        let mut slot = context.component_mut::<NFusionSlot>(context.entity(slot_id)?)?;
        slot.actions.start = start;
        slot.actions.length = length;
        Ok(())
    }

    fn handle_change_trigger(
        fusion_id: u64,
        trigger: u64,
        context: &mut Context,
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
