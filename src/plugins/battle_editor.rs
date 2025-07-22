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

        {
            let is_left = world.resource::<BattleEditorState>().is_left_team;

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
        let is_left = world.resource::<BattleEditorState>().is_left_team;

        let team_entity = {
            let data = world.resource::<BattleData>();
            if is_left {
                data.team_left
            } else {
                data.team_right
            }
        };

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                ui.heading(if is_left { "Left Team" } else { "Right Team" });
                ui.separator();

                if let Ok(team) = context.get::<NTeam>(team_entity) {
                    if ui.button(&format!("Team #{}", team.id)).clicked() {
                        action = Some(BattleEditorAction::SetCurrent(BattleEditorNode::Team(
                            team.id,
                        )));
                    }
                } else {
                    ui.label("Team not found");
                }

                Ok(())
            });
        });

        Ok((action, false))
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

                    if let Ok(team) = context.get::<NTeam>(entity) {
                        let mut team_data = team.clone();
                        ui.group(|ui| {
                            if team_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            team_data.clone().unpack_entity(context, entity).log();
                        }
                    }

                    ui.separator();

                    let (team_id, team_owner) = if let Ok(team) = context.get_by_id::<NTeam>(id) {
                        (team.id, team.owner)
                    } else {
                        (0, 0)
                    };

                    if let Ok(team) = context.get_by_id::<NTeam>(id) {
                        ui.collapsing("Houses", |ui| {
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

                    if add_house {
                        let mut new_house = NHouse::default();
                        new_house.id = next_id();
                        new_house.owner = team_owner;
                        new_house.house_name = format!("House {}", new_house.id);

                        let mut color = NHouseColor::default();
                        color.id = next_id();
                        color.owner = team_owner;
                        color.color = HexColor("#808080".to_string());

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
                } else {
                    ui.label("Team not found");
                    if ui.button("Add Default Team").clicked() {
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
        let mut add_action = false;
        let mut add_status = false;
        let mut add_color = false;
        let mut house_owner = 0u64;

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(entity) = context.entity(id) {
                    let (house_name, house_owner_temp) =
                        if let Ok(house) = context.get::<NHouse>(entity) {
                            (house.house_name.clone(), house.owner)
                        } else {
                            (String::new(), 0)
                        };
                    house_owner = house_owner_temp;

                    ui.heading(format!("House: {}", house_name));
                    ui.separator();

                    if let Ok(house) = context.get::<NHouse>(entity).cloned() {
                        let mut house_data = house.clone();

                        // Collect all component data as owned values first
                        let color_data = house.color_load(context).ok().and_then(|c| {
                            context.entity(c.id()).ok().and_then(|e| {
                                context
                                    .get::<NHouseColor>(e)
                                    .cloned()
                                    .ok()
                                    .map(|data| (e, data))
                            })
                        });
                        let action_data = house.action_load(context).ok().and_then(|a| {
                            context.entity(a.id()).ok().and_then(|e| {
                                context
                                    .get::<NActionAbility>(e)
                                    .cloned()
                                    .ok()
                                    .map(|data| (e, data))
                            })
                        });
                        let status_data = house.status_load(context).ok().and_then(|s| {
                            context.entity(s.id()).ok().and_then(|e| {
                                context
                                    .get::<NStatusAbility>(e)
                                    .cloned()
                                    .ok()
                                    .map(|data| (e, data))
                            })
                        });

                        ui.group(|ui| {
                            if house_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            house_data.clone().unpack_entity(context, entity).log();
                        }

                        ui.separator();

                        ui.collapsing("Color", |ui| {
                            if let Some((color_entity, mut color)) = color_data.clone() {
                                ui.group(|ui| {
                                    if color.view_mut(ViewContext::new(ui), context, ui).changed {
                                        color.clone().unpack_entity(context, color_entity).log();
                                    }
                                });
                            } else {
                                ui.label("Color not set");
                                if ui.button("➕ Add Default Color").clicked() {
                                    add_color = true;
                                }
                            }
                        });

                        ui.collapsing("Abilities", |ui| {
                            ui.label("House can have either Action Ability OR Status Ability:");

                            if let Some((action_entity, mut action)) = action_data.clone() {
                                ui.label("Action Ability:");
                                ui.group(|ui| {
                                    if action.view_mut(ViewContext::new(ui), context, ui).changed {
                                        action.clone().unpack_entity(context, action_entity).log();
                                    }
                                });

                                // Show inner components of action ability
                                if let Ok(action_ref) = house.action_load(context) {
                                    let action_desc_data =
                                        action_ref.description_load(context).ok().and_then(|d| {
                                            context.entity(d.id()).ok().and_then(|e| {
                                                context
                                                    .get::<NActionDescription>(e)
                                                    .cloned()
                                                    .ok()
                                                    .map(|data| (e, data))
                                            })
                                        });

                                    ui.collapsing("Action Description", |ui| {
                                        if let Some((desc_entity, mut desc)) =
                                            action_desc_data.clone()
                                        {
                                            ui.group(|ui| {
                                                if desc
                                                    .view_mut(ViewContext::new(ui), context, ui)
                                                    .changed
                                                {
                                                    desc.clone()
                                                        .unpack_entity(context, desc_entity)
                                                        .log();
                                                }
                                            });

                                            // Show action effect
                                            if let Some((_, desc_data)) = action_desc_data.clone() {
                                                let effect_data = desc_data.effect_load(context).ok().and_then(|e| {
                                                    context.entity(e.id()).ok().and_then(|ent| {
                                                        context.get::<NActionEffect>(ent).cloned().ok().map(|data| (ent, data))
                                                    })
                                                });

                                                ui.collapsing("Action Effect", |ui| {
                                                    if let Some((effect_entity, mut effect)) = effect_data.clone() {
                                                        ui.group(|ui| {
                                                            if effect.view_mut(ViewContext::new(ui), context, ui).changed {
                                                                effect.clone().unpack_entity(context, effect_entity).log();
                                                            }
                                                        });
                                                    } else {
                                                        ui.label("Action Effect not set");
                                                        if ui.button("➕ Add Default Action Effect").clicked() {
                                                            Self::create_and_link_component::<NActionEffect>(context, desc_entity, house_owner);
                                                        }
                                                    }
                                                });
                                            }
                                        } else {
                                            ui.label("Action Description not set");
                                            if ui.button("➕ Add Default Action Description").clicked() {
                                                Self::create_and_link_component::<NActionDescription>(context, action_entity, house_owner);
                                            }
                                        }
                                    });
                                }

                                if ui.button("🗑 Remove Action Ability").clicked() {
                                    // Will be handled below
                                }
                            } else if let Some((status_entity, mut status)) = status_data.clone() {
                                ui.label("Status Ability:");
                                ui.group(|ui| {
                                    if status.view_mut(ViewContext::new(ui), context, ui).changed {
                                        status.clone().unpack_entity(context, status_entity).log();
                                    }
                                });

                                // Show inner components of status ability
                                if let Ok(status_ref) = house.status_load(context) {
                                    let status_desc_data =
                                        status_ref.description_load(context).ok().and_then(|d| {
                                            context.entity(d.id()).ok().and_then(|e| {
                                                context
                                                    .get::<NStatusDescription>(e)
                                                    .cloned()
                                                    .ok()
                                                    .map(|data| (e, data))
                                            })
                                        });
                                    let status_rep_data = status_ref
                                        .representation_load(context)
                                        .ok()
                                        .and_then(|r| {
                                            context.entity(r.id()).ok().and_then(|e| {
                                                context
                                                    .get::<NStatusRepresentation>(e)
                                                    .cloned()
                                                    .ok()
                                                    .map(|data| (e, data))
                                            })
                                        });

                                    ui.collapsing("Status Description", |ui| {
                                        if let Some((desc_entity, mut desc)) =
                                            status_desc_data.clone()
                                        {
                                            ui.group(|ui| {
                                                if desc
                                                    .view_mut(ViewContext::new(ui), context, ui)
                                                    .changed
                                                {
                                                    desc.clone()
                                                        .unpack_entity(context, desc_entity)
                                                        .log();
                                                }
                                            });

                                            // Show status behavior
                                            if let Some((_, desc_data)) = status_desc_data.clone() {
                                                let behavior_data = desc_data.behavior_load(context).ok().and_then(|b| {
                                                    context.entity(b.id()).ok().and_then(|ent| {
                                                        context.get::<NStatusBehavior>(ent).cloned().ok().map(|data| (ent, data))
                                                    })
                                                });

                                                ui.collapsing("Status Behavior", |ui| {
                                                    if let Some((behavior_entity, mut behavior)) = behavior_data.clone() {
                                                        ui.group(|ui| {
                                                            if behavior.view_mut(ViewContext::new(ui), context, ui).changed {
                                                                behavior.clone().unpack_entity(context, behavior_entity).log();
                                                            }
                                                        });
                                                    } else {
                                                        ui.label("Status Behavior not set");
                                                        if ui.button("➕ Add Default Status Behavior").clicked() {
                                                            Self::create_and_link_component::<NStatusBehavior>(context, desc_entity, house_owner);
                                                        }
                                                    }
                                                });
                                            }
                                        } else {
                                            ui.label("Status Description not set");
                                            if ui.button("➕ Add Default Status Description").clicked() {
                                                Self::create_and_link_component::<NStatusDescription>(context, status_entity, house_owner);
                                            }
                                        }
                                    });

                                    ui.collapsing("Status Representation", |ui| {
                                        if let Some((rep_entity, mut rep)) = status_rep_data.clone()
                                        {
                                            ui.group(|ui| {
                                                if rep
                                                    .view_mut(ViewContext::new(ui), context, ui)
                                                    .changed
                                                {
                                                    rep.clone()
                                                        .unpack_entity(context, rep_entity)
                                                        .log();
                                                }
                                            });
                                        } else {
                                            ui.label("Status Representation not set");
                                            if ui.button("➕ Add Default Status Representation").clicked() {
                                                Self::create_and_link_component::<NStatusRepresentation>(context, status_entity, house_owner);
                                            }
                                        }
                                    });
                                }

                                if ui.button("🗑 Remove Status Ability").clicked() {
                                    // Will be handled below
                                }
                            } else {
                                ui.label("No ability set");
                                ui.horizontal(|ui| {
                                    if ui.button("➕ Add Action Ability").clicked() {
                                        add_action = true;
                                    }
                                    if ui.button("➕ Add Status Ability").clicked() {
                                        add_status = true;
                                    }
                                });
                            }
                        });
                    }

                    ui.separator();
                    ui.heading("Units");

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

                    if add_color {
                        println!("Adding color to house");
                        Self::create_and_link_component::<NHouseColor>(
                            context,
                            entity,
                            house_owner,
                        );
                        changed = true;
                    }

                    if add_action {
                        println!("Adding action ability to house");
                        Self::create_and_link_component::<NActionAbility>(
                            context,
                            entity,
                            house_owner,
                        );
                        changed = true;
                    }

                    if add_status {
                        println!("Adding status ability to house");
                        Self::create_and_link_component::<NStatusAbility>(
                            context,
                            entity,
                            house_owner,
                        );
                        changed = true;
                    }
                } else {
                    ui.label("House not found");
                    if ui.button("Add Default House").clicked() {
                        changed = true;
                    }
                }

                if add_unit {
                    Self::create_default_unit(context, id, house_owner);
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
        let mut add_representation = false;
        let mut add_behavior = false;

        world.resource_scope(|_world, mut data: Mut<BattleData>| {
            let _ = Context::from_world_r(&mut data.teams_world, |context| {
                if let Ok(entity) = context.entity(id) {
                    let unit_name = context
                        .get::<NUnit>(entity)
                        .ok()
                        .map(|u| u.unit_name.clone())
                        .unwrap_or_default();
                    ui.heading(format!("Unit: {}", unit_name));
                    ui.separator();

                    if let Ok(unit) = context.get::<NUnit>(entity).cloned() {
                        let mut unit_data = unit.clone();
                        let unit_owner = unit.owner;

                        // Collect all component data as owned values first
                        let desc_data = unit.description_load(context).ok().and_then(|d| {
                            context.entity(d.id()).ok().and_then(|e| {
                                context
                                    .get::<NUnitDescription>(e)
                                    .cloned()
                                    .ok()
                                    .map(|data| (e, data))
                            })
                        });
                        let rep_data = desc_data.as_ref().and_then(|(_, desc)| {
                            desc.representation_load(context).ok().and_then(|r| {
                                context.entity(r.id()).ok().and_then(|e| {
                                    context
                                        .get::<NUnitRepresentation>(e)
                                        .cloned()
                                        .ok()
                                        .map(|data| (e, data))
                                })
                            })
                        });
                        let behavior_data = desc_data.as_ref().and_then(|(_, desc)| {
                            desc.behavior_load(context).ok().and_then(|b| {
                                context.entity(b.id()).ok().and_then(|e| {
                                    context
                                        .get::<NUnitBehavior>(e)
                                        .cloned()
                                        .ok()
                                        .map(|data| (e, data))
                                })
                            })
                        });
                        let stats_data = unit.stats_load(context).ok().and_then(|s| {
                            context.entity(s.id()).ok().and_then(|e| {
                                context
                                    .get::<NUnitStats>(e)
                                    .cloned()
                                    .ok()
                                    .map(|data| (e, data))
                            })
                        });
                        let state_data = unit.state_load(context).ok().and_then(|s| {
                            context.entity(s.id()).ok().and_then(|e| {
                                context
                                    .get::<NUnitState>(e)
                                    .cloned()
                                    .ok()
                                    .map(|data| (e, data))
                            })
                        });

                        ui.group(|ui| {
                            if unit_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            unit_data.clone().unpack_entity(context, entity).log();
                        }

                        ui.separator();

                        ui.collapsing("Description", |ui| {
                            if let Some((desc_entity, mut desc)) = desc_data.clone() {
                                ui.group(|ui| {
                                    if desc.view_mut(ViewContext::new(ui), context, ui).changed {
                                        desc.clone().unpack_entity(context, desc_entity).log();
                                    }
                                });

                                ui.collapsing("Representation", |ui| {
                                    if let Some((rep_entity, mut rep)) = rep_data.clone() {
                                        ui.group(|ui| {
                                            if rep
                                                .view_mut(ViewContext::new(ui), context, ui)
                                                .changed
                                            {
                                                rep.clone()
                                                    .unpack_entity(context, rep_entity)
                                                    .log();
                                            }
                                        });
                                    } else {
                                        ui.label("Representation not set");
                                        if ui.button("➕ Add Default Representation").clicked() {
                                            add_representation = true;
                                        }
                                    }
                                });

                                ui.collapsing("Behavior", |ui| {
                                    if let Some((behavior_entity, mut behavior)) =
                                        behavior_data.clone()
                                    {
                                        ui.group(|ui| {
                                            if behavior
                                                .view_mut(ViewContext::new(ui), context, ui)
                                                .changed
                                            {
                                                behavior
                                                    .clone()
                                                    .unpack_entity(context, behavior_entity)
                                                    .log();
                                            }
                                        });
                                    } else {
                                        ui.label("Behavior not set");
                                        if ui.button("➕ Add Default Behavior").clicked() {
                                            add_behavior = true;
                                        }
                                    }
                                });

                                if add_representation {
                                    println!("Adding representation to unit description");
                                    Self::create_and_link_component::<NUnitRepresentation>(
                                        context,
                                        desc_entity,
                                        unit_owner,
                                    );
                                    changed = true;
                                }

                                if add_behavior {
                                    println!("Adding behavior to unit description");
                                    Self::create_and_link_component::<NUnitBehavior>(
                                        context,
                                        desc_entity,
                                        unit_owner,
                                    );
                                    changed = true;
                                }
                            } else {
                                ui.label("Description not set");
                            }
                        });

                        ui.collapsing("Stats", |ui| {
                            if let Some((stats_entity, mut stats)) = stats_data.clone() {
                                ui.group(|ui| {
                                    if stats.view_mut(ViewContext::new(ui), context, ui).changed {
                                        stats.clone().unpack_entity(context, stats_entity).log();
                                    }
                                });
                            } else {
                                ui.label("Stats not set");
                                if ui.button("➕ Add Default Stats").clicked() {
                                    Self::create_and_link_component::<NUnitStats>(
                                        context, entity, unit_owner,
                                    );
                                    changed = true;
                                }
                            }
                        });

                        ui.collapsing("State", |ui| {
                            if let Some((state_entity, mut state)) = state_data.clone() {
                                ui.group(|ui| {
                                    if state.view_mut(ViewContext::new(ui), context, ui).changed {
                                        state.clone().unpack_entity(context, state_entity).log();
                                    }
                                });
                            } else {
                                ui.label("State not set");
                                if ui.button("➕ Add Default State").clicked() {
                                    Self::create_and_link_component::<NUnitState>(
                                        context, entity, unit_owner,
                                    );
                                    changed = true;
                                }
                            }
                        });
                    }
                } else {
                    ui.label("Unit not found");
                    if ui.button("Add Default Unit").clicked() {
                        changed = true;
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

                    if let Ok(fusion) = context.get::<NFusion>(entity).cloned() {
                        let mut fusion_data = fusion.clone();
                        ui.group(|ui| {
                            if fusion_data
                                .view_mut(ViewContext::new(ui), context, ui)
                                .changed
                            {
                                changed = true;
                            }
                        });

                        if changed {
                            fusion_data.clone().unpack_entity(context, entity).log();
                        }

                        ui.separator();

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
                } else {
                    ui.label("Fusion not found");
                    if ui.button("Add Default Fusion").clicked() {
                        changed = true;
                    }
                }

                Ok(())
            });
        });

        Ok((action, changed))
    }

    fn create_and_link_component<T>(context: &mut Context, parent_entity: Entity, owner: u64)
    where
        T: Node + 'static,
    {
        let mut component = T::default();
        component.set_owner(owner);
        let component_entity = context.world_mut().unwrap().spawn_empty().id();

        component.unpack_entity(context, component_entity).log();
        context
            .link_parent_child_entity(parent_entity, component_entity)
            .log();
        let component_id = context.id(component_entity).unwrap();
        println!(
            "Creating component {} with ID {} for parent entity {:?}",
            std::any::type_name::<T>(),
            component_id,
            parent_entity
        );
        // Update parent references based on component type
        Self::update_parent_component_reference::<T>(
            context,
            parent_entity,
            component_id,
            component_entity,
        );
    }

    fn update_parent_component_reference<T>(
        context: &mut Context,
        parent_entity: Entity,
        component_id: u64,
        component_entity: Entity,
    ) where
        T: Node + 'static,
    {
        let type_name = std::any::type_name::<T>();
        println!("Updating parent reference for type: {}", type_name);

        // Handle NActionAbility for houses
        if type_name.contains("NActionAbility") {
            if let Ok(mut house) = context.get::<NHouse>(parent_entity).cloned() {
                house.action = Some(NActionAbility {
                    id: component_id,
                    owner: house.owner,
                    entity: Some(component_entity),
                    ability_name: String::new(),
                    description: None,
                });
                house.unpack_entity(context, parent_entity).log();
                println!("Updated house with action ability");
            }
        }
        // Handle NHouseColor for houses
        else if type_name.contains("NHouseColor") {
            if let Ok(mut house) = context.get::<NHouse>(parent_entity).cloned() {
                house.color = Some(NHouseColor {
                    id: component_id,
                    owner: house.owner,
                    entity: Some(component_entity),
                    color: HexColor("#808080".to_string()),
                });
                house.unpack_entity(context, parent_entity).log();
                println!("Updated house with color");
            }
        }
        // Handle NStatusAbility for houses
        else if type_name.contains("NStatusAbility") {
            if let Ok(mut house) = context.get::<NHouse>(parent_entity).cloned() {
                house.status = Some(NStatusAbility {
                    id: component_id,
                    owner: house.owner,
                    entity: Some(component_entity),
                    status_name: String::new(),
                    description: None,
                    representation: None,
                });
                house.unpack_entity(context, parent_entity).log();
                println!("Updated house with status ability");
            }
        }
        // Handle NUnitRepresentation for unit descriptions
        else if type_name.contains("NUnitRepresentation") {
            if let Ok(mut desc) = context.get::<NUnitDescription>(parent_entity).cloned() {
                desc.representation = Some(NUnitRepresentation {
                    id: component_id,
                    owner: desc.owner,
                    entity: Some(component_entity),
                    material: Material::default(),
                });
                desc.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NUnitBehavior for unit descriptions
        else if type_name.contains("NUnitBehavior") {
            if let Ok(mut desc) = context.get::<NUnitDescription>(parent_entity).cloned() {
                desc.behavior = Some(NUnitBehavior {
                    id: component_id,
                    owner: desc.owner,
                    entity: Some(component_entity),
                    reactions: vec![],
                });
                desc.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NActionDescription for action abilities
        else if type_name.contains("NActionDescription") {
            if let Ok(mut action) = context.get::<NActionAbility>(parent_entity).cloned() {
                action.description = Some(NActionDescription {
                    id: component_id,
                    owner: action.owner,
                    entity: Some(component_entity),
                    description: String::new(),
                    effect: None,
                });
                action.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NActionEffect for action descriptions
        else if type_name.contains("NActionEffect") {
            if let Ok(mut desc) = context.get::<NActionDescription>(parent_entity).cloned() {
                desc.effect = Some(NActionEffect {
                    id: component_id,
                    owner: desc.owner,
                    entity: Some(component_entity),
                    actions: vec![],
                });
                desc.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NStatusDescription for status abilities
        else if type_name.contains("NStatusDescription") {
            if let Ok(mut status) = context.get::<NStatusAbility>(parent_entity).cloned() {
                status.description = Some(NStatusDescription {
                    id: component_id,
                    owner: status.owner,
                    entity: Some(component_entity),
                    description: String::new(),
                    behavior: None,
                });
                status.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NStatusBehavior for status descriptions
        else if type_name.contains("NStatusBehavior") {
            if let Ok(mut desc) = context.get::<NStatusDescription>(parent_entity).cloned() {
                desc.behavior = Some(NStatusBehavior {
                    id: component_id,
                    owner: desc.owner,
                    entity: Some(component_entity),
                    reactions: vec![],
                });
                desc.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NStatusRepresentation for status abilities
        else if type_name.contains("NStatusRepresentation") {
            if let Ok(mut status) = context.get::<NStatusAbility>(parent_entity).cloned() {
                status.representation = Some(NStatusRepresentation {
                    id: component_id,
                    owner: status.owner,
                    entity: Some(component_entity),
                    material: Material::default(),
                });
                status.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NUnitStats for units
        else if type_name.contains("NUnitStats") {
            if let Ok(mut unit) = context.get::<NUnit>(parent_entity).cloned() {
                unit.stats = Some(NUnitStats {
                    id: component_id,
                    owner: unit.owner,
                    entity: Some(component_entity),
                    pwr: 1,
                    hp: 1,
                });
                unit.unpack_entity(context, parent_entity).log();
            }
        }
        // Handle NUnitState for units
        else if type_name.contains("NUnitState") {
            if let Ok(mut unit) = context.get::<NUnit>(parent_entity).cloned() {
                unit.state = Some(NUnitState {
                    id: component_id,
                    owner: unit.owner,
                    entity: Some(component_entity),
                    xp: 0,
                    lvl: 1,
                    rarity: 0,
                });
                unit.unpack_entity(context, parent_entity).log();
            }
        }
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
