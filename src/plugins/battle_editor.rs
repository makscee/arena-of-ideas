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
    Fusion(u64),
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

        // Set up slot actions
        if let Some(mut battle_data) = world.remove_resource::<BattleData>() {
            battle_data.slot_actions = vec![(
                "Add default unit".to_string(),
                Self::handle_add_default_unit,
            )];
            world.insert_resource(battle_data);
        }

        {
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

                    navigation_action = Some(BattleEditorAction::SetCurrent(
                        BattleEditorNode::Team(team_id),
                    ));
                }
                if ui.selectable_label(!is_left, "Right Team").clicked() {
                    world.resource_mut::<BattleEditorState>().is_left_team = false;

                    let mut battle_data = world.remove_resource::<BattleData>().unwrap();
                    let team_id = Context::from_world_r(&mut battle_data.teams_world, |context| {
                        context.id(battle_data.team_right)
                    })
                    .unwrap_or(0);
                    world.insert_resource(battle_data);

                    navigation_action = Some(BattleEditorAction::SetCurrent(
                        BattleEditorNode::Team(team_id),
                    ));
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
                        BattleEditorNode::Fusion(id) => {
                            format!("[tw Fusion #{}]", id).cstr().label(ui);
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
                        BattleEditorNode::Fusion(id) => Self::render_fusion_editor(*id, ui, world),
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
        }

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
            "[h2 Houses]".cstr().label(ui);
            let (houses_changed, house_action) =
                Self::show_children_node_editors::<NHouse>(id, context, ui, id, |house_id| {
                    BattleEditorAction::Navigate(BattleEditorNode::House(house_id))
                })?;
            changed |= houses_changed;
            if house_action.is_some() {
                action = house_action;
            }
            ui.separator();
            "[h2 Fusions]".cstr().label(ui);

            let (fusions_changed, fusion_action) =
                Self::show_children_node_editors::<NFusion>(id, context, ui, id, |fusion_id| {
                    BattleEditorAction::Navigate(BattleEditorNode::Fusion(fusion_id))
                })?;
            changed |= fusions_changed;
            if fusion_action.is_some() {
                action = fusion_action;
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

    fn render_fusion_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let action = None;
        let mut changed = false;

        let mut battle_data = world.remove_resource::<BattleData>().unwrap();
        let result = Context::from_world_r(&mut battle_data.teams_world, |context| {
            if let Ok(mut fusion) = context.get::<NFusion>(context.entity(id)?).cloned() {
                let mut add_unit: Option<u64> = None;
                let mut remove_unit: Option<u64> = None;
                let units = fusion.units(context)?;
                for (slot_idx, unit) in units.iter().enumerate() {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("Unit {}", unit.id));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("Remove").clicked() {
                                            remove_unit = Some(unit.id);
                                        }
                                    },
                                );
                            });

                            ui.separator();
                            match fusion.render_unit_range_selector(ui, context, unit, slot_idx) {
                                Ok(range_changed) => {
                                    if range_changed {
                                        changed = true;
                                    }
                                }
                                Err(e) => {
                                    e.cstr().notify_error_op();
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);
                }

                ui.separator();
                ui.heading("Add Units from Houses");

                let team = context.first_parent::<NTeam>(id)?;
                let houses = team.houses_load(context);

                for house in houses {
                    ui.collapsing(format!("House: {}", house.house_name), |ui| {
                        let house_units = house.units_load(context);
                        for unit in house_units {
                            if !units.iter().any(|u| u.id == unit.id) {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Unit: {}", unit.id));
                                    if ui.button("Add to Fusion").clicked() {
                                        add_unit = Some(unit.id);
                                    }
                                });
                            }
                        }
                    });
                }

                if let Some(unit_id) = add_unit {
                    context.link_parent_child(unit_id, id).notify_error_op();
                    fusion.units.ids.push(unit_id);
                    fusion.action_limit = 100;
                    changed = true;
                }

                if let Some(unit_id) = remove_unit {
                    fusion.remove_unit(unit_id)?;
                    context
                        .unlink_parent_child(unit_id, fusion.id)
                        .notify_error_op();
                    changed = true;
                }
                if changed {
                    let entity = fusion.entity();
                    fusion.unpack_entity(context, entity).notify_error_op();
                }
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
            let mut node = context.get_mut::<T>(entity).unwrap();
            let id = node.id();
            let owner = node.owner();
            *node = data;
            node.set_id(id);
            node.set_owner(owner);
            node.set_entity(entity);
            changed = true;
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
                    fusion.slot = 0;
                    fusion.pwr = 1;
                    fusion.hp = 1;
                    fusion.dmg = 1;
                    fusion.lvl = 1;
                    fusion.action_limit = 1;
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

    fn handle_add_default_unit(slot: i32, team_entity: Entity, context: &mut Context) {
        // Fusion slots are always 0-4 regardless of which team
        let fusion_slot = slot.abs() - 1; // Convert 1-5 or -1 to -5 into 0-4

        // Get team and find existing fusion for this slot
        let (team_owner, fusion_id) = {
            let team = match context.get::<NTeam>(team_entity) {
                Ok(team) => team,
                Err(_) => return,
            };
            let fusion = team.fusions.iter().find(|f| f.slot == fusion_slot);
            match fusion {
                Some(f) => (team.owner, f.id),
                None => return, // Should never happen - teams should always have 5 fusions
            }
        };

        let fusion_entity = context.entity(fusion_id).unwrap_or(Entity::PLACEHOLDER);

        // Create a new house
        let house_id = Self::create_node::<NHouse>(context, team_owner);
        let house_entity = context.entity(house_id).unwrap_or(Entity::PLACEHOLDER);

        // Create a new unit
        let unit_id = Self::create_node::<NUnit>(context, team_owner);
        let unit_entity = context.entity(unit_id).unwrap_or(Entity::PLACEHOLDER);

        // Set unit properties
        if let Ok(mut unit) = context.get_mut::<NUnit>(unit_entity) {
            unit.unit_name = "Default Unit".to_string();
        }

        // Get unit for adding to house
        let unit_clone = context
            .get::<NUnit>(unit_entity)
            .cloned()
            .unwrap_or_default();

        // Add unit to house
        if let Ok(mut house) = context.get_mut::<NHouse>(house_entity) {
            house.units.push(unit_clone);
        }

        // Add unit to fusion
        if let Ok(mut fusion) = context.get_mut::<NFusion>(fusion_entity) {
            fusion.units.ids.push(unit_id);
        }

        // Update team fusions list
        let fusion_id = context.id(fusion_entity).unwrap_or(0);
        let updated_fusion = context.get::<NFusion>(fusion_entity).cloned().ok();
        if let (Ok(mut team), Some(fusion)) =
            (context.get_mut::<NTeam>(team_entity), updated_fusion)
        {
            if let Some(fusion_in_team) = team.fusions.iter_mut().find(|f| f.id == fusion_id) {
                *fusion_in_team = fusion;
            }
        }

        op(|world| {
            Self::save_team_changes(world);
        });
    }
}

enum BattleEditorAction {
    GoBack,
    Navigate(BattleEditorNode),
    SetCurrent(BattleEditorNode),
}
