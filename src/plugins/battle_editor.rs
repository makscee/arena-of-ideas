use super::*;
use crate::nodes::*;
use crate::resources::*;
use crate::ui::*;
use crate::utils::*;
use bevy_egui::egui::ScrollArea;

pub struct BattleEditorPlugin;

impl Plugin for BattleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleEditorState>()
            .add_systems(OnEnter(GameState::Loaded), Self::init);
    }
}

#[derive(Resource, Default)]
pub struct BattleEditorState {
    pub current_node: Option<BattleEditorNode>,
    pub navigation_stack: Vec<BattleEditorNode>,
    pub is_left_team: bool,
}

#[derive(Clone, Debug)]
pub enum BattleEditorNode {
    Team(u64),
    House(u64),
    Unit(u64),
    Fusion(u64),
}

impl BattleEditorPlugin {
    fn init(mut state: ResMut<BattleEditorState>) {
        state.current_node = None;
        state.navigation_stack.clear();
        state.is_left_team = true;
    }

    pub fn pane(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut navigation_action: Option<BattleEditorAction> = None;
        let mut changed = false;

        // Render UI and collect navigation actions
        {
            let is_left = world.resource::<BattleEditorState>().is_left_team;

            // Team selector
            ui.horizontal(|ui| {
                ui.label("Editing:");
                if ui.selectable_label(is_left, "Left Team").clicked() {
                    world.resource_mut::<BattleEditorState>().is_left_team = true;
                    navigation_action = Some(BattleEditorAction::Reset);
                }
                if ui.selectable_label(!is_left, "Right Team").clicked() {
                    world.resource_mut::<BattleEditorState>().is_left_team = false;
                    navigation_action = Some(BattleEditorAction::Reset);
                }
            });

            ui.separator();

            // Navigation bar
            let state = world.resource::<BattleEditorState>();
            ui.horizontal(|ui| {
                if !state.navigation_stack.is_empty() {
                    if ui.button("← Back").clicked() {
                        navigation_action = Some(BattleEditorAction::GoBack);
                    }
                    ui.separator();
                }

                // Show current node path
                if let Some(current) = &state.current_node {
                    match current {
                        BattleEditorNode::Team(id) => {
                            ui.label(format!("Team #{}", id));
                        }
                        BattleEditorNode::House(id) => {
                            ui.label(format!("House #{}", id));
                        }
                        BattleEditorNode::Unit(id) => {
                            ui.label(format!("Unit #{}", id));
                        }
                        BattleEditorNode::Fusion(id) => {
                            ui.label(format!("Fusion #{}", id));
                        }
                    }
                } else {
                    ui.label("Select a component to edit");
                }
            });

            ui.separator();

            // Main content area
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
                    _ => {}
                }
            });
        }

        // Apply navigation action if any
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
                BattleEditorAction::Reset => {
                    state.current_node = None;
                    state.navigation_stack.clear();
                }
            }
        }

        // If any changes were made, request reload
        if changed {
            world.resource_mut::<ReloadData>().reload_requested = true;
        }

        Ok(())
    }

    fn render_team_selector(
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let mut changed = false;
        let mut add_house = false;
        let mut team_id = 0u64;
        let mut team_owner = 0u64;

        world.resource_scope(|world, mut data: Mut<BattleData>| {
            let is_left = world.resource::<BattleEditorState>().is_left_team;
            let team_entity = if is_left {
                data.team_left
            } else {
                data.team_right
            };

            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(team) = context.get::<NTeam>(team_entity) {
                    ui.heading(format!("{} Team", if is_left { "Left" } else { "Right" }));
                    ui.separator();

                    // Store team data before any UI operations
                    team_id = team.id;
                    team_owner = team.owner;

                    // Edit the team directly
                    if let Ok(mut team_data) = NTeam::pack_entity(context, team_entity) {
                        ui.group(|ui| {
                            if team_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            team_data.unpack_entity(context, team_entity).log();
                        }
                    }

                    ui.separator();

                    // Navigation to children
                    if let Ok(team) = context.get::<NTeam>(team_entity) {
                        // Houses section
                        ui.collapsing("Houses", |ui| {
                            // Add new house button
                            if ui.button("➕ Add New House").clicked() {
                                add_house = true;
                            }

                            ui.separator();

                            let houses = team.houses_load(context);
                            for house in houses {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("Edit {}", house.house_name)).clicked() {
                                        action = Some(BattleEditorAction::SetCurrent(
                                            BattleEditorNode::House(house.id()),
                                        ));
                                    }

                                    // Show house preview
                                    if let Ok(color) = house.color_load(context) {
                                        let egui_color = egui::Color32::from_hex(&color.color.0)
                                            .unwrap_or(egui::Color32::WHITE);
                                        ui.colored_label(egui_color, "■");
                                    }

                                    let units = house.units_load(context);
                                    ui.label(format!("Units: {}", units.len()));
                                });
                            }
                        });

                        // Fusions section
                        ui.collapsing("Fusions", |ui| {
                            let fusions = team.fusions_load(context);
                            for fusion in fusions {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("Edit Fusion #{}", fusion.id())).clicked()
                                    {
                                        action = Some(BattleEditorAction::SetCurrent(
                                            BattleEditorNode::Fusion(fusion.id()),
                                        ));
                                    }

                                    ui.label(format!(
                                        "Units: {}, Slot: {}, Level: {}",
                                        fusion.units.ids.len(),
                                        fusion.slot,
                                        fusion.lvl
                                    ));
                                });
                            }
                        });
                    }
                }

                // Create new house after UI if needed
                if add_house {
                    let mut new_house = NHouse::default();
                    new_house.id = next_id();
                    new_house.owner = team_owner;
                    new_house.house_name = format!("House {}", new_house.id);

                    // Create default color
                    let mut color = NHouseColor::default();
                    color.id = next_id();
                    color.owner = team_owner;
                    color.color = HexColor("#808080".to_string());

                    // Create and link entities
                    let house_entity = context.world_mut().unwrap().spawn_empty().id();
                    let color_entity = context.world_mut().unwrap().spawn_empty().id();

                    let color_id = color.id;
                    let house_id = new_house.id;

                    color.unpack_entity(context, color_entity).log();
                    new_house.color = Some(NHouseColor {
                        id: color_id,
                        owner: team_owner,
                        entity: Some(color_entity),
                        color: HexColor("#808080".to_string()),
                    });
                    new_house.unpack_entity(context, house_entity).log();

                    context
                        .link_parent_child_entity(house_entity, color_entity)
                        .log();
                    context.link_parent_child(team_id, house_id).log();

                    changed = true;
                }

                Ok(())
            });
        });

        Ok((action, changed))
    }

    fn render_team_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let mut changed = false;
        let mut add_house = false;

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(entity) = context.entity(id) {
                    ui.heading("Team Editor");
                    ui.separator();

                    // Edit team
                    if let Ok(mut team_data) = NTeam::pack_entity(context, entity) {
                        ui.group(|ui| {
                            if team_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            team_data.unpack_entity(context, entity).log();
                        }
                    }

                    ui.separator();

                    // Store team data before UI operations
                    let (team_id, team_owner) = if let Ok(team) = context.get_by_id::<NTeam>(id) {
                        (team.id, team.owner)
                    } else {
                        (0, 0)
                    };

                    // Navigation to children
                    if let Ok(team) = context.get_by_id::<NTeam>(id) {
                        // Houses section
                        ui.collapsing("Houses", |ui| {
                            // Add new house button
                            if ui.button("➕ Add New House").clicked() {
                                add_house = true;
                            }

                            ui.separator();

                            let houses = team.houses_load(context);
                            for house in houses {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("Edit {}", house.house_name)).clicked() {
                                        action = Some(BattleEditorAction::Navigate(
                                            BattleEditorNode::House(house.id()),
                                        ));
                                    }

                                    // Show house preview
                                    if let Ok(color) = house.color_load(context) {
                                        let egui_color = egui::Color32::from_hex(&color.color.0)
                                            .unwrap_or(egui::Color32::WHITE);
                                        ui.colored_label(egui_color, "■");
                                    }

                                    let units = house.units_load(context);
                                    ui.label(format!("Units: {}", units.len()));
                                });
                            }
                        });

                        // Fusions section
                        ui.collapsing("Fusions", |ui| {
                            let fusions = team.fusions_load(context);
                            for fusion in fusions {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("Edit Fusion #{}", fusion.id())).clicked()
                                    {
                                        action = Some(BattleEditorAction::Navigate(
                                            BattleEditorNode::Fusion(fusion.id()),
                                        ));
                                    }

                                    ui.label(format!(
                                        "Units: {}, Slot: {}, Level: {}",
                                        fusion.units.ids.len(),
                                        fusion.slot,
                                        fusion.lvl
                                    ));
                                });
                            }
                        });
                    }

                    // Create new house after UI if needed
                    if add_house {
                        let mut new_house = NHouse::default();
                        new_house.id = next_id();
                        new_house.owner = team_owner;
                        new_house.house_name = format!("House {}", new_house.id);

                        // Create default color
                        let mut color = NHouseColor::default();
                        color.id = next_id();
                        color.owner = team_owner;
                        color.color = HexColor("#808080".to_string());

                        // Create and link entities
                        let house_entity = context.world_mut().unwrap().spawn_empty().id();
                        let color_entity = context.world_mut().unwrap().spawn_empty().id();

                        let color_id = color.id;
                        let house_id = new_house.id;

                        color.unpack_entity(context, color_entity).log();
                        new_house.color = Some(NHouseColor {
                            id: color_id,
                            owner: team_owner,
                            entity: Some(color_entity),
                            color: HexColor("#808080".to_string()),
                        });
                        new_house.unpack_entity(context, house_entity).log();

                        context
                            .link_parent_child_entity(house_entity, color_entity)
                            .log();
                        context.link_parent_child(team_id, house_id).log();

                        changed = true;
                    }
                }

                Ok(())
            });
        });

        Ok((action, changed))
    }

    fn render_house_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let mut changed = false;
        let mut add_unit = false;
        let mut house_owner = 0u64;

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(entity) = context.entity(id) {
                    // Get house data first
                    let (house_name, house_owner_temp) =
                        if let Ok(house) = context.get::<NHouse>(entity) {
                            (house.house_name.clone(), house.owner)
                        } else {
                            (String::new(), 0)
                        };
                    house_owner = house_owner_temp;

                    ui.heading(format!("House: {}", house_name));
                    ui.separator();

                    if let Ok(mut house_data) = NHouse::pack_entity(context, entity) {
                        ui.group(|ui| {
                            if house_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            house_data.unpack_entity(context, entity).log();
                        }
                    }

                    ui.separator();

                    // Get child component IDs first to avoid borrow conflicts
                    let (color_id, action_id, status_id) =
                        if let Ok(house) = context.get::<NHouse>(entity) {
                            (
                                house.color.as_ref().map(|c| c.id),
                                house.action.as_ref().map(|a| a.id),
                                house.status.as_ref().map(|s| s.id),
                            )
                        } else {
                            (None, None, None)
                        };

                    // Color
                    if let Some(color_id) = color_id {
                        ui.collapsing("Color", |ui| {
                            if let Ok(color_entity) = context.entity(color_id) {
                                if let Ok(mut color_data) =
                                    NHouseColor::pack_entity(context, color_entity)
                                {
                                    ui.group(|ui| {
                                        if color_data
                                            .view_mut(ViewContext::new(ui), context, ui)
                                            .changed
                                        {
                                            changed = true;
                                        }
                                    });

                                    if changed {
                                        color_data.unpack_entity(context, color_entity).log();
                                    }
                                }
                            }
                        });
                    }

                    // Action ability
                    if let Some(action_id) = action_id {
                        ui.collapsing("Action Ability", |ui| {
                            if let Ok(action_entity) = context.entity(action_id) {
                                if let Ok(mut action_data) =
                                    NActionAbility::pack_entity(context, action_entity)
                                {
                                    ui.group(|ui| {
                                        if action_data
                                            .view_mut(ViewContext::new(ui), context, ui)
                                            .changed
                                        {
                                            changed = true;
                                        }
                                    });

                                    if changed {
                                        action_data.unpack_entity(context, action_entity).log();
                                    }
                                }
                            }
                        });
                    }

                    // Status ability
                    if let Some(status_id) = status_id {
                        ui.collapsing("Status Ability", |ui| {
                            if let Ok(status_entity) = context.entity(status_id) {
                                if let Ok(mut status_data) =
                                    NStatusAbility::pack_entity(context, status_entity)
                                {
                                    ui.group(|ui| {
                                        if status_data
                                            .view_mut(ViewContext::new(ui), context, ui)
                                            .changed
                                        {
                                            changed = true;
                                        }
                                    });

                                    if changed {
                                        status_data.unpack_entity(context, status_entity).log();
                                    }
                                }
                            }
                        });
                    }

                    ui.separator();

                    // Units section
                    ui.heading("Units");

                    // Add new unit button
                    if ui.button("➕ Add New Unit").clicked() {
                        add_unit = true;
                    }

                    ui.separator();

                    if let Ok(house) = context.get_by_id::<NHouse>(id) {
                        let units = house.units_load(context);
                        for unit in units {
                            ui.horizontal(|ui| {
                                if ui.button(format!("Edit {}", unit.unit_name)).clicked() {
                                    action = Some(BattleEditorAction::Navigate(
                                        BattleEditorNode::Unit(unit.id()),
                                    ));
                                }

                                // Show unit preview
                                if let Ok(stats) = unit.stats_load(context) {
                                    ui.label(format!("PWR: {}, HP: {}", stats.pwr, stats.hp));
                                }

                                if let Ok(state) = unit.state_load(context) {
                                    ui.label(format!(
                                        "Lvl: {}, XP: {}, Rarity: {}",
                                        state.lvl, state.xp, state.rarity
                                    ));
                                }
                            });
                        }
                    }
                }

                // Create new unit after UI if needed
                if add_unit {
                    let mut new_unit = NUnit::default();
                    new_unit.id = next_id();
                    new_unit.owner = house_owner;
                    new_unit.unit_name = format!("Unit {}", new_unit.id);

                    // Create default components
                    let mut stats = NUnitStats::default();
                    stats.id = next_id();
                    stats.owner = house_owner;
                    stats.pwr = 1;
                    stats.hp = 1;

                    let mut state = NUnitState::default();
                    state.id = next_id();
                    state.owner = house_owner;
                    state.lvl = 1;
                    state.xp = 0;
                    state.rarity = 0;

                    let mut description = NUnitDescription::default();
                    description.id = next_id();
                    description.owner = house_owner;

                    // Create entities
                    let unit_entity = context.world_mut().unwrap().spawn_empty().id();
                    let stats_entity = context.world_mut().unwrap().spawn_empty().id();
                    let state_entity = context.world_mut().unwrap().spawn_empty().id();
                    let desc_entity = context.world_mut().unwrap().spawn_empty().id();

                    let stats_id = stats.id;
                    let state_id = state.id;
                    let desc_id = description.id;
                    let unit_id = new_unit.id;

                    stats.unpack_entity(context, stats_entity).log();
                    state.unpack_entity(context, state_entity).log();
                    description.unpack_entity(context, desc_entity).log();

                    // Set references
                    new_unit.stats = Some(NUnitStats {
                        id: stats_id,
                        owner: house_owner,
                        entity: Some(stats_entity),
                        pwr: 1,
                        hp: 1,
                    });
                    new_unit.state = Some(NUnitState {
                        id: state_id,
                        owner: house_owner,
                        entity: Some(state_entity),
                        lvl: 1,
                        xp: 0,
                        rarity: 0,
                    });
                    new_unit.description = Some(NUnitDescription {
                        id: desc_id,
                        owner: house_owner,
                        entity: Some(desc_entity),
                        description: String::new(),
                        representation: None,
                        behavior: None,
                    });
                    new_unit.unpack_entity(context, unit_entity).log();

                    // Link entities
                    context
                        .link_parent_child_entity(unit_entity, stats_entity)
                        .log();
                    context
                        .link_parent_child_entity(unit_entity, state_entity)
                        .log();
                    context
                        .link_parent_child_entity(unit_entity, desc_entity)
                        .log();
                    context.link_parent_child(id, unit_id).log();

                    changed = true;
                }

                Ok(())
            });
        });

        Ok((action, changed))
    }

    fn render_unit_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let action = None;
        let mut changed = false;

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(entity) = context.entity(id) {
                    // Get unit name first
                    let unit_name = context
                        .get::<NUnit>(entity)
                        .ok()
                        .map(|u| u.unit_name.clone())
                        .unwrap_or_default();
                    ui.heading(format!("Unit: {}", unit_name));
                    ui.separator();

                    // Edit unit
                    if let Ok(mut unit_data) = NUnit::pack_entity(context, entity) {
                        ui.group(|ui| {
                            if unit_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            unit_data.unpack_entity(context, entity).log();
                        }
                    }

                    ui.separator();

                    // Get child component IDs first to avoid borrow conflicts
                    let (desc_id, stats_id, state_id) =
                        if let Ok(unit) = context.get::<NUnit>(entity) {
                            (
                                unit.description.as_ref().map(|d| d.id),
                                unit.stats.as_ref().map(|s| s.id),
                                unit.state.as_ref().map(|s| s.id),
                            )
                        } else {
                            (None, None, None)
                        };

                    // Description and its children
                    if let Some(desc_id) = desc_id {
                        ui.collapsing("Description", |ui| {
                            if let Ok(desc_entity) = context.entity(desc_id) {
                                // First, get the child IDs
                                let (rep_id, behavior_id) = if let Ok(desc) =
                                    context.get::<NUnitDescription>(desc_entity)
                                {
                                    (
                                        desc.representation.as_ref().map(|r| r.id),
                                        desc.behavior.as_ref().map(|b| b.id),
                                    )
                                } else {
                                    (None, None)
                                };

                                // Now edit the description
                                if let Ok(mut desc_data) =
                                    NUnitDescription::pack_entity(context, desc_entity)
                                {
                                    ui.group(|ui| {
                                        if desc_data
                                            .view_mut(ViewContext::new(ui), context, ui)
                                            .changed
                                        {
                                            changed = true;
                                        }
                                    });

                                    if changed {
                                        desc_data.unpack_entity(context, desc_entity).log();
                                    }
                                }

                                // Representation
                                if let Some(rep_id) = rep_id {
                                    ui.collapsing("Representation", |ui| {
                                        if let Ok(rep_entity) = context.entity(rep_id) {
                                            if let Ok(mut rep_data) =
                                                NUnitRepresentation::pack_entity(
                                                    context, rep_entity,
                                                )
                                            {
                                                ui.group(|ui| {
                                                    if rep_data
                                                        .view_mut(ViewContext::new(ui), context, ui)
                                                        .changed
                                                    {
                                                        changed = true;
                                                    }
                                                });

                                                if changed {
                                                    rep_data
                                                        .unpack_entity(context, rep_entity)
                                                        .log();
                                                }
                                            }
                                        }
                                    });
                                }

                                // Behavior
                                if let Some(behavior_id) = behavior_id {
                                    ui.collapsing("Behavior", |ui| {
                                        if let Ok(behavior_entity) = context.entity(behavior_id) {
                                            if let Ok(mut behavior_data) =
                                                NUnitBehavior::pack_entity(context, behavior_entity)
                                            {
                                                ui.group(|ui| {
                                                    if behavior_data
                                                        .view_mut(ViewContext::new(ui), context, ui)
                                                        .changed
                                                    {
                                                        changed = true;
                                                    }
                                                });

                                                if changed {
                                                    behavior_data
                                                        .unpack_entity(context, behavior_entity)
                                                        .log();
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    }

                    // Stats
                    if let Some(stats_id) = stats_id {
                        ui.collapsing("Stats", |ui| {
                            if let Ok(stats_entity) = context.entity(stats_id) {
                                if let Ok(mut stats_data) =
                                    NUnitStats::pack_entity(context, stats_entity)
                                {
                                    ui.group(|ui| {
                                        if stats_data
                                            .view_mut(ViewContext::new(ui), context, ui)
                                            .changed
                                        {
                                            changed = true;
                                        }
                                    });

                                    if changed {
                                        stats_data.unpack_entity(context, stats_entity).log();
                                    }
                                }
                            }
                        });
                    }

                    // State
                    if let Some(state_id) = state_id {
                        ui.collapsing("State", |ui| {
                            if let Ok(state_entity) = context.entity(state_id) {
                                if let Ok(mut state_data) =
                                    NUnitState::pack_entity(context, state_entity)
                                {
                                    ui.group(|ui| {
                                        if state_data
                                            .view_mut(ViewContext::new(ui), context, ui)
                                            .changed
                                        {
                                            changed = true;
                                        }
                                    });

                                    if changed {
                                        state_data.unpack_entity(context, state_entity).log();
                                    }
                                }
                            }
                        });
                    }
                }
                Ok(())
            });
        });

        Ok((action, changed))
    }

    fn render_fusion_editor(
        id: u64,
        ui: &mut Ui,
        world: &mut World,
    ) -> Result<(Option<BattleEditorAction>, bool), ExpressionError> {
        let mut action = None;
        let mut changed = false;

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(entity) = context.entity(id) {
                    ui.heading("Fusion Editor");
                    ui.separator();

                    // Edit fusion
                    if let Ok(mut fusion_data) = NFusion::pack_entity(context, entity) {
                        ui.group(|ui| {
                            if fusion_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            fusion_data.unpack_entity(context, entity).log();
                        }
                    }

                    ui.separator();

                    // Navigation to linked units
                    if let Ok(fusion) = context.get_by_id::<NFusion>(id) {
                        ui.heading("Linked Units");
                        for unit_id in &fusion.units.ids {
                            if let Ok(unit) = context.get_by_id::<NUnit>(*unit_id) {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("View {}", unit.unit_name)).clicked() {
                                        action = Some(BattleEditorAction::Navigate(
                                            BattleEditorNode::Unit(*unit_id),
                                        ));
                                    }
                                });
                            }
                        }
                    }
                }
                Ok(())
            });
        });

        Ok((action, changed))
    }
}

enum BattleEditorAction {
    GoBack,
    Navigate(BattleEditorNode),
    SetCurrent(BattleEditorNode),
    Reset,
}
