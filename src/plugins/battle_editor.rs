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
    fn init(world: &mut World) {
        world.init_resource::<BattleEditorState>();
    }

    pub fn pane(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut navigation_action: Option<BattleEditorAction> = None;
        let mut changed = false;

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
                let team_id = team.id();
                ui.label(format!("Team: {}", team.id));
                if ui.button("Edit Team").clicked() {
                    action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::Team(
                        context.id(team_entity)?,
                    )));
                }

                ui.separator();
                ui.heading("Houses");

                let houses = team.houses_load(context);
                let mut houses_to_delete = Vec::new();
                for house in houses {
                    ui.horizontal(|ui| {
                        if ui.button(format!("Edit {}", house.house_name)).clicked() {
                            action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::House(
                                house.id,
                            )));
                        }

                        if ui.button("🗑 Delete").clicked() {
                            houses_to_delete.push(house.entity());
                        }
                    });
                }

                for entity in houses_to_delete {
                    context.despawn(entity).log();
                    changed = true;
                }

                if ui.button("➕ Add Empty House").clicked() {
                    let mut house = NHouse::default();
                    house.house_name = "New House".into();
                    let entity = context.world_mut()?.spawn_empty().id();
                    house.unpack_entity(context, entity)?;
                    context.link_parent_child(team_id, context.id(entity)?)?;
                    changed = true;
                }
            }
            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
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
            if let Ok(entity) = context.entity(id) {
                if let Ok(mut team) = context.get::<NTeam>(entity).cloned() {
                    ui.heading(format!("Team: {}", team.id));
                    ui.separator();

                    ui.group(|ui| {
                        if team.view_mut(ViewContext::new(ui), context, ui).changed {
                            team.clone().unpack_entity(context, entity).log();
                            changed = true;
                        }
                    });

                    ui.separator();
                    ui.heading("Houses");

                    if ui.button("➕ Add Empty House").clicked() {
                        Self::create_and_link_component::<NHouse>(context, entity, team.id);
                        changed = true;
                    }

                    ui.separator();

                    let houses = team.houses_load(context);
                    let mut houses_to_delete = Vec::new();
                    for house in houses {
                        ui.horizontal(|ui| {
                            if ui.button(format!("Edit {}", house.house_name)).clicked() {
                                action = Some(BattleEditorAction::Navigate(
                                    BattleEditorNode::House(house.id()),
                                ));
                            }

                            if ui.button("🗑 Delete").clicked() {
                                houses_to_delete.push(house.entity());
                            }
                        });
                    }

                    for entity in houses_to_delete {
                        context.despawn(entity).log();
                        changed = true;
                    }

                    ui.separator();
                    ui.heading("Fusions");

                    let fusions = team.fusions_load(context);
                    let mut fusions_to_delete = Vec::new();
                    for fusion in fusions {
                        ui.horizontal(|ui| {
                            if ui.button("Edit Fusion").clicked() {
                                action = Some(BattleEditorAction::Navigate(
                                    BattleEditorNode::Fusion(fusion.id()),
                                ));
                            }

                            if ui.button("🗑 Delete").clicked() {
                                fusions_to_delete.push(fusion.entity());
                            }
                        });
                    }

                    for entity in fusions_to_delete {
                        context.despawn(entity).log();
                        changed = true;
                    }

                    if ui.button("➕ Add Fusion").clicked() {
                        Self::create_and_link_component::<NFusion>(context, entity, team.id);
                        changed = true;
                    }
                } else {
                    ui.label("Team not found");
                }
            } else {
                ui.label("Team entity not found");
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
            if let Ok(entity) = context.entity(id) {
                if let Ok(mut house) = context.get::<NHouse>(entity).cloned() {
                    ui.heading(format!("House: {}", house.house_name));
                    ui.separator();

                    ui.group(|ui| {
                        if house.view_mut(ViewContext::new(ui), context, ui).changed {
                            house.clone().unpack_entity(context, entity).log();
                            changed = true;
                        }
                    });

                    ui.separator();

                    ui.collapsing("Color", |ui| {
                        if let Ok((color_changed, _)) =
                            Self::show_node_editor::<NHouseColor>(entity, context, ui, house.owner)
                        {
                            changed |= color_changed;
                        }
                    });

                    ui.collapsing("Abilities", |ui| {
                        ui.label("House can have either Action Ability OR Status Ability:");

                        ui.collapsing("Action Ability", |ui| {
                            if let Ok((action_changed, action_entity)) =
                                Self::show_node_editor::<NActionAbility>(
                                    entity,
                                    context,
                                    ui,
                                    house.owner,
                                )
                            {
                                changed |= action_changed;

                                if let Some(action_entity) = action_entity {
                                    ui.collapsing("Action Description", |ui| {
                                        if let Ok((desc_changed, desc_entity)) =
                                            Self::show_node_editor::<NActionDescription>(
                                                action_entity,
                                                context,
                                                ui,
                                                house.owner,
                                            )
                                        {
                                            changed |= desc_changed;

                                            if let Some(desc_entity) = desc_entity {
                                                ui.collapsing("Action Effect", |ui| {
                                                    if let Ok((effect_changed, _)) =
                                                        Self::show_node_editor::<NActionEffect>(
                                                            desc_entity,
                                                            context,
                                                            ui,
                                                            house.owner,
                                                        )
                                                    {
                                                        changed |= effect_changed;
                                                    }
                                                });
                                            }
                                        }
                                    });
                                }
                            }
                        });

                        ui.collapsing("Status Ability", |ui| {
                            if let Ok((status_changed, status_entity)) =
                                Self::show_node_editor::<NStatusAbility>(
                                    entity,
                                    context,
                                    ui,
                                    house.owner,
                                )
                            {
                                changed |= status_changed;

                                if let Some(status_entity) = status_entity {
                                    ui.collapsing("Status Description", |ui| {
                                        if let Ok((desc_changed, desc_entity)) =
                                            Self::show_node_editor::<NStatusDescription>(
                                                status_entity,
                                                context,
                                                ui,
                                                house.owner,
                                            )
                                        {
                                            changed |= desc_changed;

                                            if let Some(desc_entity) = desc_entity {
                                                ui.collapsing("Status Behavior", |ui| {
                                                    if let Ok((behavior_changed, _)) =
                                                        Self::show_node_editor::<NStatusBehavior>(
                                                            desc_entity,
                                                            context,
                                                            ui,
                                                            house.owner,
                                                        )
                                                    {
                                                        changed |= behavior_changed;
                                                    }
                                                });
                                            }
                                        }
                                    });

                                    ui.collapsing("Status Representation", |ui| {
                                        if let Ok((repr_changed, _)) =
                                            Self::show_node_editor::<NStatusRepresentation>(
                                                status_entity,
                                                context,
                                                ui,
                                                house.owner,
                                            )
                                        {
                                            changed |= repr_changed;
                                        }
                                    });
                                }
                            }
                        });
                    });

                    ui.separator();
                    ui.heading("Units");

                    if ui.button("➕ Add New Unit").clicked() {
                        Self::create_default_unit(context, id, house.owner);
                        changed = true;
                    }

                    ui.separator();

                    let units = house.units_load(context);
                    let mut units_to_delete = Vec::new();
                    for unit in units {
                        ui.horizontal(|ui| {
                            if ui.button(format!("Edit {}", unit.unit_name)).clicked() {
                                action = Some(BattleEditorAction::Navigate(
                                    BattleEditorNode::Unit(unit.id()),
                                ));
                            }

                            if let Ok(stats) = unit.stats_load(context) {
                                ui.label(format!("PWR: {}, HP: {}", stats.pwr, stats.hp));
                            }

                            if let Ok(state) = unit.state_load(context) {
                                ui.label(format!(
                                    "Lvl: {}, XP: {}, Rarity: {}",
                                    state.lvl, state.xp, state.rarity
                                ));
                            }

                            if ui.button("🗑 Delete").clicked() {
                                units_to_delete.push(unit.entity());
                            }
                        });
                    }

                    for entity in units_to_delete {
                        context.despawn(entity).log();
                        changed = true;
                    }
                } else {
                    ui.label("House not found");
                    if ui.button("Add Default House").clicked() {
                        changed = true;
                    }
                }
            } else {
                ui.label("House entity not found");
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
            if let Ok(entity) = context.entity(id) {
                if let Ok(mut unit) = context.get::<NUnit>(entity).cloned() {
                    ui.heading(format!("Unit: {}", unit.unit_name));
                    ui.separator();

                    ui.group(|ui| {
                        if unit.view_mut(ViewContext::new(ui), context, ui).changed {
                            unit.clone().unpack_entity(context, entity).log();
                            changed = true;
                        }
                    });

                    ui.separator();

                    ui.collapsing("Stats", |ui| {
                        if let Ok((stats_changed, _)) =
                            Self::show_node_editor::<NUnitStats>(entity, context, ui, unit.owner)
                        {
                            changed |= stats_changed;
                        }
                    });

                    ui.collapsing("State", |ui| {
                        if let Ok((state_changed, _)) =
                            Self::show_node_editor::<NUnitState>(entity, context, ui, unit.owner)
                        {
                            changed |= state_changed;
                        }
                    });

                    ui.collapsing("Description", |ui| {
                        if let Ok((desc_changed, desc_entity)) =
                            Self::show_node_editor::<NUnitDescription>(
                                entity, context, ui, unit.owner,
                            )
                        {
                            changed |= desc_changed;

                            if let Some(desc_entity) = desc_entity {
                                ui.collapsing("Unit Behavior", |ui| {
                                    if let Ok((behavior_changed, _)) =
                                        Self::show_node_editor::<NUnitBehavior>(
                                            desc_entity,
                                            context,
                                            ui,
                                            unit.owner,
                                        )
                                    {
                                        changed |= behavior_changed;
                                    }
                                });

                                ui.collapsing("Unit Representation", |ui| {
                                    if let Ok((repr_changed, _)) =
                                        Self::show_node_editor::<NUnitRepresentation>(
                                            desc_entity,
                                            context,
                                            ui,
                                            unit.owner,
                                        )
                                    {
                                        changed |= repr_changed;
                                    }
                                });
                            }
                        }
                    });

                    ui.separator();
                } else {
                    ui.label("Unit not found");
                }
            } else {
                ui.label("Unit entity not found");
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
            if let Ok(entity) = context.entity(id) {
                if let Ok(mut fusion) = context.get::<NFusion>(entity).cloned() {
                    ui.heading("Fusion");
                    ui.separator();

                    ui.group(|ui| {
                        if fusion.view_mut(ViewContext::new(ui), context, ui).changed {
                            fusion.clone().unpack_entity(context, entity).log();
                            changed = true;
                        }
                    });

                    ui.separator();
                } else {
                    ui.label("Fusion not found");
                }
            } else {
                ui.label("Fusion entity not found");
            }
            Ok(())
        });

        world.insert_resource(battle_data);
        result?;
        Ok((action, changed))
    }

    fn show_node_editor<T>(
        child_entity: Entity,
        context: &mut Context,
        ui: &mut Ui,
        owner: u64,
    ) -> Result<(bool, Option<Entity>), ExpressionError>
    where
        T: Node + 'static + View,
    {
        let mut changed = false;
        let mut found_entity = None;

        if let Ok(mut node) = context
            .first_parent::<T>(context.id(child_entity)?)
            .cloned()
        {
            let entity = node.entity();
            found_entity = Some(entity);

            ui.group(|ui| {
                if node.view_mut(ViewContext::new(ui), context, ui).changed {
                    changed = true;
                }
            });

            if changed {
                node.unpack_entity(context, entity).log();
            }

            if ui
                .button(format!("🗑 Remove {}", T::kind_s().cstr()))
                .clicked()
            {
                context.despawn(entity).log();
                changed = true;
                found_entity = None;
            }
        } else {
            ui.label(format!("{} not set", T::kind_s().cstr()));
            if ui
                .button(format!("➕ Add Default {}", T::kind_s().cstr()))
                .clicked()
            {
                Self::create_and_link_component::<T>(context, child_entity, owner);
                changed = true;
            }
        }

        Ok((changed, found_entity))
    }

    fn create_and_link_component<T>(context: &mut Context, child_entity: Entity, owner: u64)
    where
        T: Node + 'static,
    {
        let mut component = T::default();
        component.set_owner(owner);
        let component_entity = context.world_mut().unwrap().spawn_empty().id();

        component.unpack_entity(context, component_entity).log();
        context
            .link_parent_child_entity(component_entity, child_entity)
            .log();
    }

    fn create_default_unit(context: &mut Context, house_id: u64, owner: u64) {
        let mut new_unit = NUnit::default();
        new_unit.id = next_id();
        new_unit.owner = owner;
        new_unit.unit_name = format!("Unit {}", new_unit.id);

        let mut stats = NUnitStats::default();
        stats.id = next_id();
        stats.owner = owner;
        stats.pwr = 1;
        stats.hp = 1;

        let mut state = NUnitState::default();
        state.id = next_id();
        state.owner = owner;
        state.lvl = 1;
        state.xp = 0;
        state.rarity = 0;

        let mut description = NUnitDescription::default();
        description.id = next_id();
        description.owner = owner;

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

        new_unit.stats = Some(NUnitStats {
            id: stats_id,
            owner,
            entity: Some(stats_entity),
            pwr: 1,
            hp: 1,
        });
        new_unit.state = Some(NUnitState {
            id: state_id,
            owner,
            entity: Some(state_entity),
            lvl: 1,
            xp: 0,
            rarity: 0,
        });
        new_unit.description = Some(NUnitDescription {
            id: desc_id,
            owner,
            entity: Some(desc_entity),
            description: String::new(),
            representation: None,
            behavior: None,
        });
        new_unit.unpack_entity(context, unit_entity).log();

        context
            .link_parent_child_entity(unit_entity, stats_entity)
            .log();
        context
            .link_parent_child_entity(unit_entity, state_entity)
            .log();
        context
            .link_parent_child_entity(unit_entity, desc_entity)
            .log();
        context.link_parent_child(house_id, unit_id).log();
    }
}

enum BattleEditorAction {
    GoBack,
    Navigate(BattleEditorNode),
    SetCurrent(BattleEditorNode),
    Reset,
}
