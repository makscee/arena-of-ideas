use super::*;
use crate::nodes::*;
use crate::resources::*;
use crate::ui::render::*;
use crate::ui::*;
use crate::utils::*;
use bevy::ecs::component::Mutable;
use bevy_egui::egui::ScrollArea;

pub struct BattleEditorV2Plugin;

impl Plugin for BattleEditorV2Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleEditorV2State>()
            .add_systems(OnEnter(GameState::Loaded), Self::init);
    }
}

#[derive(Resource)]
pub struct BattleEditorV2State {
    pub current_node: Option<BattleEditorNode>,
    pub navigation_stack: Vec<BattleEditorNode>,
    pub is_left_team: bool,
}

impl Default for BattleEditorV2State {
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

impl BattleEditorV2Plugin {
    fn init(world: &mut World) {
        let mut state = world.resource_mut::<BattleEditorV2State>();
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

        let is_left = world.resource::<BattleEditorV2State>().is_left_team;

        // Team selector header
        ui.horizontal(|ui| {
            ui.label("Editing:");
            if ui.selectable_label(is_left, "Left Team").clicked() {
                world.resource_mut::<BattleEditorV2State>().is_left_team = true;

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
                world.resource_mut::<BattleEditorV2State>().is_left_team = false;

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
        let state = world.resource::<BattleEditorV2State>();
        ui.horizontal(|ui| {
            if !state.navigation_stack.is_empty() {
                if ui.button("← Back").clicked() {
                    navigation_action = Some(BattleEditorAction::GoBack);
                }
                ui.separator();
            }

            // Show current node using render system
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

        // Main content area
        let current_node = world.resource::<BattleEditorV2State>().current_node.clone();
        ScrollArea::vertical().show(ui, |ui| {
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
        });

        // Handle navigation actions
        if let Some(action) = navigation_action {
            let mut state = world.resource_mut::<BattleEditorV2State>();
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
        let is_left = world.resource::<BattleEditorV2State>().is_left_team;

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
            for house in &houses {
                ui.horizontal(|ui| {
                    // Use render system with context menu
                    let response = house
                        .render(context)
                        .with_menu()
                        .add_copy()
                        .add_paste()
                        .add_delete()
                        .tag(ui);

                    if ui.button("Edit").clicked() {
                        action = Some(BattleEditorAction::Navigate(BattleEditorNode::House(
                            house.id(),
                        )));
                    }

                    if let Some(deleted) = response.deleted() {
                        context.despawn(house.entity()).log();
                        changed = true;
                    }
                });
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
            let house = context.component::<NHouse>(entity)?;

            // Use render system for house display
            ui.group(|ui| {
                house.render(context).card(ui);
            });

            ui.separator();

            // Edit house name
            ui.horizontal(|ui| {
                ui.label("House Name:");
                let mut house_mut = context.component_mut::<NHouse>(entity)?;
                if ui.text_edit_singleline(&mut house_mut.house_name).changed() {
                    changed = true;
                }
            });

            ui.separator();

            // House color
            if let Ok(color) = context.first_parent::<NHouseColor>(id) {
                ui.group(|ui| {
                    color.render(context).title(ui);
                    color.render(context).display(ui);
                });
            }

            // Abilities and statuses using render system
            if let Ok(ability) = house.ability_load(context) {
                ui.separator();
                ui.label("Ability:");
                ability.render(context).tag_card(ui);
            }

            if let Ok(status) = house.status_load(context) {
                ui.separator();
                ui.label("Status:");
                status.render(context).tag_card(ui);
            }

            ui.separator();
            "[h2 Units]".cstr().label(ui);

            // Render units using the new render system
            let units = context.collect_children_components::<NUnit>(id)?;
            for unit in &units {
                ui.horizontal(|ui| {
                    let response = unit
                        .render(context)
                        .with_menu()
                        .add_copy()
                        .add_paste()
                        .add_delete()
                        .tag(ui);

                    if ui.button("Edit").clicked() {
                        action = Some(BattleEditorAction::Navigate(BattleEditorNode::Unit(
                            unit.id(),
                        )));
                    }

                    if let Some(deleted) = response.deleted() {
                        context.despawn(unit.entity()).log();
                        changed = true;
                    }
                });
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
            let unit = context.component::<NUnit>(entity)?;

            // Use render system for unit display
            ui.group(|ui| {
                unit.render(context).card(ui);
            });

            ui.separator();

            // Edit unit name
            ui.horizontal(|ui| {
                ui.label("Unit Name:");
                let mut unit_mut = context.component_mut::<NUnit>(entity)?;
                if ui.text_edit_singleline(&mut unit_mut.unit_name).changed() {
                    changed = true;
                }
            });

            ui.separator();

            // Stats using render system
            if let Ok(stats) = unit.stats_load(context) {
                ui.group(|ui| {
                    stats.render(context).title(ui);
                    stats.render(context).display(ui);

                    // Edit stats
                    ui.horizontal(|ui| {
                        ui.label("Power:");
                        let mut stats_mut = context.component_mut::<NUnitStats>(stats.entity())?;
                        if ui.add(egui::DragValue::new(&mut stats_mut.pwr)).changed() {
                            changed = true;
                        }
                        ui.label("HP:");
                        if ui.add(egui::DragValue::new(&mut stats_mut.hp)).changed() {
                            changed = true;
                        }
                    });
                });
            }

            // State
            if let Ok(state) = unit.state_load(context) {
                ui.separator();
                ui.group(|ui| {
                    state.render(context).title(ui);
                    state.render(context).display(ui);
                });
            }

            // Description
            if let Ok(desc) = unit.description_load(context) {
                ui.separator();
                ui.group(|ui| {
                    desc.render(context).display(ui);

                    // Behavior
                    if let Ok(behavior) = desc.behavior_load(context) {
                        ui.separator();
                        behavior.render(context).display(ui);
                    }

                    // Representation
                    if let Ok(repr) = desc.representation_load(context) {
                        ui.separator();
                        repr.render(context).tag(ui);
                    }
                });
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
