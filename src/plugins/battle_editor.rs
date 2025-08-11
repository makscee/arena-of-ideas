use super::*;
use crate::nodes::*;
use crate::resources::*;
use crate::ui::*;
use crate::utils::*;
use bevy::ecs::component::Mutable;
use bevy_egui::egui::ScrollArea;

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
                state.current_node =
                    Some(BattleEditorNode::Team(pd().client_state.battle_test.0.root))
            }
        }
    }

    pub fn pane(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut navigation_action: Option<BattleEditorAction> = None;
        let mut changed = false;

        let is_left = world.resource::<BattleEditorState>().is_left_team;

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

        let state = world.resource::<BattleEditorState>();
        ui.horizontal(|ui| {
            if !state.navigation_stack.is_empty() {
                if ui.button("← Back").clicked() {
                    navigation_action = Some(BattleEditorAction::GoBack);
                }
                ui.separator();
            }

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

        ui.separator();

        let current_node = world.resource::<BattleEditorState>().current_node.clone();
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
                    err.ui(ui);
                }
                _ => {}
            }
        });

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

            if let Ok(team) = context.get::<NTeam>(team_entity) {
                ui.label(format!("Team: {}", team.id));
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
            let team_entity = context.entity(id)?;

            "[h2 Team Editor]".cstr().label(ui);

            const ACTION_DEFAULT_UNIT: &str = "Add Default Unit";
            const ACTION_UNIT_EDITOR: &str = "Open Unit Editor";
            const ACTION_DELETE_UNIT: &str = "Delete Unit";
            let team_editor = TeamEditor::new(team_entity)
                .empty_slot_action(ACTION_DEFAULT_UNIT)
                .filled_slot_action(ACTION_UNIT_EDITOR)
                .filled_slot_action(ACTION_DELETE_UNIT);
            let actions = team_editor.ui(ui, context)?;

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
                }
            }

            ui.separator();
            "[h2 Houses]".cstr().label(ui);
            let (houses_changed, house_action) =
                Self::show_children_node_editors::<NHouse>(id, context, ui, id, |house_id| {
                    BattleEditorAction::Navigate(BattleEditorNode::House(house_id))
                })?;
            changed |= houses_changed;
            if house_action.is_some() {
                action = house_action;
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
            if Self::show_node_editor::<NHouse>(id, context, ui)? {
                changed = true;
            }
            ui.separator();

            Self::show_parent_node_editor::<NHouseColor>(id, context, ui, id, &mut changed);

            if let Some(action_id) =
                Self::show_parent_node_editor::<NActionAbility>(id, context, ui, id, &mut changed)
            {
                if let Some(desc_id) = Self::show_parent_node_editor::<NActionDescription>(
                    action_id,
                    context,
                    ui,
                    id,
                    &mut changed,
                ) {
                    Self::show_parent_node_editor::<NActionEffect>(
                        desc_id,
                        context,
                        ui,
                        id,
                        &mut changed,
                    );
                }
            }

            if let Some(status_id) =
                Self::show_parent_node_editor::<NStatusAbility>(id, context, ui, id, &mut changed)
            {
                if let Some(desc_id) = Self::show_parent_node_editor::<NStatusDescription>(
                    status_id,
                    context,
                    ui,
                    id,
                    &mut changed,
                ) {
                    Self::show_parent_node_editor::<NStatusBehavior>(
                        desc_id,
                        context,
                        ui,
                        id,
                        &mut changed,
                    );

                    Self::show_parent_node_editor::<NStatusRepresentation>(
                        status_id,
                        context,
                        ui,
                        id,
                        &mut changed,
                    );
                }
            }

            ui.separator();
            "[h2 Units]".cstr().label(ui);

            let (units_changed, unit_action) =
                Self::show_children_node_editors::<NUnit>(id, context, ui, id, |unit_id| {
                    BattleEditorAction::Navigate(BattleEditorNode::Unit(unit_id))
                })?;

            changed |= units_changed;
            if unit_action.is_some() {
                action = unit_action;
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
            ui.heading(format!("Unit: {}", id));
            ui.separator();

            if Self::show_node_editor::<NUnit>(id, context, ui)? {
                changed = true;
            }

            ui.separator();

            Self::show_parent_node_editor::<NUnitStats>(id, context, ui, id, &mut changed);
            Self::show_parent_node_editor::<NUnitState>(id, context, ui, id, &mut changed);

            if let Some(desc_id) =
                Self::show_parent_node_editor::<NUnitDescription>(id, context, ui, id, &mut changed)
            {
                Self::show_parent_node_editor::<NUnitRepresentation>(
                    desc_id,
                    context,
                    ui,
                    id,
                    &mut changed,
                );

                Self::show_parent_node_editor::<NUnitBehavior>(
                    desc_id,
                    context,
                    ui,
                    id,
                    &mut changed,
                );
            }

            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn show_node_editor<T>(
        id: u64,
        context: &mut Context,
        ui: &mut Ui,
    ) -> Result<bool, ExpressionError>
    where
        T: Node + 'static + View + SFnInfo,
    {
        let mut changed = false;

        let entity = context.entity(id)?;
        let mut node = context.get::<T>(entity).cloned().map_err(|_| {
            ui.label(format!("{} not found", T::kind_s().cstr()));
            ExpressionError::from("Node not found")
        })?;
        ui.group(|ui| {
            node.see(context).info().label(ui);
            changed |= node.show_mut(context, ui);
        });
        if changed {
            node.unpack_entity(context, entity).log();
        }
        Ok(changed)
    }

    fn show_parent_node_editor<T>(
        child_id: u64,
        context: &mut Context,
        ui: &mut Ui,
        owner: u64,
        changed: &mut bool,
    ) -> Option<u64>
    where
        T: Node + 'static + SFnInfo + SFnTitle,
    {
        let mut parent_id = None;
        if let Ok(parent) = context.first_parent::<T>(child_id) {
            let parent_entity = parent.entity();
            parent_id = Some(parent.id());

            let mut parent_node = parent.clone();

            ui.collapsing(format!("{}", T::kind_s().cstr()), |ui| {
                let mut local_changed = false;
                let mut was_deleted = false;

                ui.horizontal(|ui| {
                    let btn_response = parent_node.see(context).node_ctxbtn_full().ui(ui);

                    parent_node.see(context).info().label(ui);

                    if btn_response.deleted() {
                        was_deleted = true;
                    }
                    if let Some(pasted_node) = btn_response.pasted_node_full() {
                        if let Err(e) = pasted_node.clone().unpack_entity(context, parent_entity) {
                            error!("Failed to unpack parent node: {}", e);
                        } else {
                            local_changed = true;
                        }
                    }
                });

                if !was_deleted {
                    ui.group(|ui| {
                        if parent_node.show_mut(context, ui) {
                            local_changed = true;
                        }
                    });
                }

                if was_deleted {
                    context.despawn(parent_entity).log();
                    *changed = true;
                    parent_id = None;
                } else if local_changed {
                    parent_node.unpack_entity(context, parent_entity).log();
                    *changed = true;
                }
            });
        } else {
            ui.label(format!("{} not set", T::kind_s().cstr()));
            if ui
                .button(format!("➕ Add Default {}", T::kind_s().cstr()))
                .clicked()
            {
                Self::create_node::<T>(context, owner)
                    .add_child(context.world_mut().unwrap(), child_id);
                *changed = true;
            }
        }

        parent_id
    }

    fn show_node_with_action_button<T>(
        node: &T,
        context: &Context,
        ui: &mut Ui,
        action_button: Option<(&str, &mut bool)>,
    ) -> bool
    where
        T: Node + SFnInfo + SFnTitle,
    {
        let mut navigate_clicked = false;

        ui.horizontal(|ui| {
            if let Some((button_text, button_clicked)) = action_button {
                if ui.button(button_text).clicked() {
                    *button_clicked = true;
                }
            }

            let btn_response = node.see(context).node_ctxbtn_full().ui(ui);
            node.see(context).info().label(ui);

            if btn_response.clicked() {
                navigate_clicked = true;
            }
        });

        navigate_clicked
    }

    fn show_children_node_editors<T>(
        parent_id: u64,
        context: &mut Context,
        ui: &mut Ui,
        owner: u64,
        action_callback: impl Fn(u64) -> BattleEditorAction,
    ) -> Result<(bool, Option<BattleEditorAction>), ExpressionError>
    where
        T: Node + 'static + Component<Mutability = Mutable> + SFnInfo + SFnTitle,
    {
        let mut changed = false;
        let mut action = None;

        let children = context.collect_children_components::<T>(parent_id)?;
        let mut children_to_delete = Vec::new();

        let mut pasted: Option<(Entity, T)> = None;
        let mut node_full_pasted: Option<(Entity, T)> = None;

        for node in children {
            ui.horizontal(|ui| {
                let btn_response = node.see(context).node_ctxbtn_full().ui(ui);
                node.see(context).info().label(ui);
                if btn_response.clicked() {
                    action = Some(action_callback(node.id()));
                }

                if btn_response.deleted() {
                    children_to_delete.push(node.entity());
                }

                if let Some(pasted_data) = btn_response.pasted() {
                    pasted = Some((node.entity(), pasted_data.clone()));
                }

                if let Some(pasted_node_full) = btn_response.pasted_node_full() {
                    node_full_pasted = Some((node.entity(), pasted_node_full.clone()));
                }
            });
        }

        if let Some((entity, data)) = pasted {
            if let Ok(mut node_mut) = context.get_mut::<T>(entity) {
                let id = node_mut.id();
                let owner = node_mut.owner();
                *node_mut = data;
                node_mut.set_id(id);
                node_mut.set_owner(owner);
                node_mut.set_entity(entity);
                changed = true;
            }
        }

        if let Some((entity, node_full_data)) = node_full_pasted {
            if let Err(e) = node_full_data.unpack_entity(context, entity) {
                error!("Failed to unpack node: {}", e);
            } else {
                changed = true;
            }
        }

        for entity in children_to_delete {
            context.despawn(entity).log();
            changed = true;
        }

        if ui
            .button(format!("➕ Add {}", T::kind_s().cstr()))
            .clicked()
        {
            Self::create_node::<T>(context, owner).add_parent(context.world_mut()?, parent_id);
            changed = true;
        }

        Ok((changed, action))
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
                    // fusion.slot = 0;
                    fusion.pwr = 1;
                    fusion.hp = 1;
                    fusion.dmg = 1;
                    // fusion.lvl = 1;
                    fusion.actions_limit = 1;
                }
            }
            _ => {}
        }
        let component_entity = context.world_mut().unwrap().spawn_empty().id();
        component.unpack_entity(context, component_entity).log();
        id
    }

    fn save_team_changes(world: &mut World) {
        world.resource_mut::<ReloadData>().reload_requested = true;
    }

    fn handle_move_unit(
        unit_id: u64,
        target_id: u64,
        context: &mut Context,
    ) -> Result<(), ExpressionError> {
        let old_slot_id = if let Ok(slot) = context.first_child::<NBenchSlot>(unit_id) {
            slot.id
        } else if let Ok(slot) = context.first_child::<NFusionSlot>(unit_id) {
            slot.id
        } else {
            return Err(
                ExpressionErrorVariants::Custom("Unit is not in a slot".to_string()).into(),
            );
        };
        context.unlink_parent_child(unit_id, old_slot_id)?;
        context.link_parent_child(unit_id, target_id)?;
        Ok(())
    }

    fn handle_add_default_unit_to_slot(
        team_entity: Entity,
        slot_id: u64,
        context: &mut Context,
    ) -> Result<(), ExpressionError> {
        let slot_entity = context.entity(slot_id)?;
        let team = context.get::<NTeam>(team_entity)?;
        let team_owner = team.owner;

        let unit_stats = NUnitStats::new_full(team_owner, 1, 1);
        let unit_state = NUnitState::new_full(team_owner, 1, 1);
        let unit_behavior = NUnitBehavior::new_full(team_owner, vec![]);
        let unit_representation = NUnitRepresentation::new_full(
            team_owner,
            Material(vec![PainterAction::circle(Box::new(Expression::f32(0.5)))]),
        );

        let unit = NUnit::new_full(
            team_owner,
            "Default Unit".to_string(),
            default(),
            NUnitDescription::new_full(
                team_owner,
                "Default Description".into(),
                unit_representation,
                unit_behavior,
            ),
            unit_stats,
            unit_state,
        );
        let unit_entity = context.world_mut()?.spawn_empty().id();
        unit.unpack_entity(context, unit_entity)?;

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
                let house = NHouse::new_full(
                    team_owner,
                    "Default House".to_string(),
                    default(),
                    house_color,
                    NActionAbility::new_full(team_owner, "Default Action".to_string(), default()),
                    NStatusAbility::new_full(
                        team_owner,
                        "Default Status".to_string(),
                        default(),
                        default(),
                    ),
                );

                let house_entity = context.world_mut()?.spawn_empty().id();
                house.clone().unpack_entity(context, house_entity)?;
                context.link_parent_child_entity(team_entity, house_entity)?;
                house_entity
            }
        };
        context.link_parent_child_entity(house_entity, unit_entity)?;
        context.link_parent_child_entity(unit_entity, slot_entity)?;

        Ok(())
    }

    fn handle_add_slot(fusion_id: u64, context: &mut Context) -> Result<(), ExpressionError> {
        let fusion_entity = context.entity(fusion_id)?;
        let fusion = context.get::<NFusion>(fusion_entity)?;

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
}

enum BattleEditorAction {
    GoBack,
    Navigate(BattleEditorNode),
    SetCurrent(BattleEditorNode),
}
