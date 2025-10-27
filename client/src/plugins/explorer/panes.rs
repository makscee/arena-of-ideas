use super::*;
use std::collections::HashMap;

pub struct ExplorerPanes;

impl ExplorerPanes {
    pub fn render_node_list(ui: &mut Ui, world: &mut World, kind: NamedNodeKind) -> NodeResult<()> {
        named_node_kind_match!(
            kind,
            if "➕ Add new".cstr().button(ui).clicked() {
                NamedNodeType::default().open_publish_window(world);
            }
        );
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();
            let names = match kind {
                NamedNodeKind::NUnit => &cache.unit_names,
                NamedNodeKind::NHouse => &cache.house_names,
                NamedNodeKind::NAbilityMagic => &cache.ability_names,
                NamedNodeKind::NStatusMagic => &cache.status_names,
            };
            let inspected_name = match kind {
                NamedNodeKind::NUnit => state.inspected_unit.clone(),
                NamedNodeKind::NHouse => state.inspected_house.clone(),
                NamedNodeKind::NAbilityMagic => state.inspected_ability.clone(),
                NamedNodeKind::NStatusMagic => state.inspected_status.clone(),
            };
            todo!("reimplement");
            // names
            //     .as_list(|name, _ctx, ui| {
            //         let color = if inspected_name
            //             .as_ref()
            //             .is_some_and(|inspected| inspected.eq(name))
            //         {
            //             YELLOW
            //         } else {
            //             colorix().high_contrast_text()
            //         };
            //         name.cstr_c(color).label(ui)
            //     })
            //     .with_hover(|name, _ctx, ui| {
            //         if ui.button("Inspect").clicked() {
            //             let action = match kind {
            //                 NamedNodeKind::NUnit => ExplorerAction::InspectUnit(name.to_string()),
            //                 NamedNodeKind::NHouse => ExplorerAction::InspectHouse(name.to_string()),
            //                 NamedNodeKind::NAbilityMagic => {
            //                     ExplorerAction::InspectAbility(name.to_string())
            //                 }
            //                 NamedNodeKind::NStatusMagic => {
            //                     ExplorerAction::InspectStatus(name.to_string())
            //                 }
            //             };

            //             state.pending_actions.push(action);
            //         }
            //     })
            //     .compose(&cn().db().as_context(), ui);

            Ok(())
        })
    }

    pub fn render_node_card<T>(node: &T, ui: &mut Ui) -> NodeResult<()>
    where
        T: ClientNode + FCard,
    {
        node.as_card().compose(&EMPTY_CONTEXT, ui);
        Ok(())
    }

    fn render_parent_link_list(
        ui: &mut Ui,
        house_names: &Vec<String>,
        current_parent: &str,
        selected_parent: &str,
        inspected_house: &Option<String>,
        child_id: u64,
        inspect_action: fn(String) -> ExplorerAction,
        cache: &ExplorerCache,
        pending_actions: &mut Vec<ExplorerAction>,
    ) -> NodeResult<()> {
        house_names
            .as_list(|house_name, _ctx, ui| {
                let mut display_parts = vec![];
                if house_name == current_parent {
                    display_parts.push("Current");
                }
                if house_name == selected_parent {
                    display_parts.push("Selected");
                }

                let display_name = if display_parts.is_empty() {
                    house_name.clone()
                } else {
                    format!("{} ({})", house_name, display_parts.join(", "))
                };

                let color = if inspected_house
                    .as_ref()
                    .is_some_and(|name| name == house_name)
                {
                    YELLOW
                } else if house_name == current_parent || house_name == selected_parent {
                    colorix().accent().text_1()
                } else {
                    colorix().high_contrast_text()
                };

                display_name.cstr_c(color).label(ui)
            })
            .with_hover(|house_name, _ctx, ui| {
                ui.horizontal(|ui| {
                    if ui.button("Inspect").clicked() {
                        pending_actions.push(inspect_action(house_name.clone()));
                    }
                    if house_name != selected_parent {
                        if ui.button("Select").clicked() {
                            if let Some(house_node) = cache.houses.get(house_name) {
                                cn().reducers
                                    .content_select_link(house_node.0.id(), child_id)
                                    .notify_op();
                            }
                        }
                    }
                });
            })
            .compose(&EMPTY_CONTEXT, ui);

        Ok(())
    }

    fn render_house_child_list_single(
        ui: &mut Ui,
        house_name: &str,
        item_names: &Vec<String>,
        item_parents: &HashMap<String, (String, String)>,
        inspected_item: &Option<String>,
        inspect_action: fn(String) -> ExplorerAction,
        is_ability: bool,
        cache: &ExplorerCache,
        pending_actions: &mut Vec<ExplorerAction>,
    ) -> NodeResult<()> {
        item_names
            .as_list(|item_name, _ctx, ui| {
                let (current_parent, selected_parent) = item_parents
                    .get(item_name)
                    .map(|(top, player)| (top.as_str(), player.as_str()))
                    .unwrap_or(("", ""));

                let mut display_parts = vec![];
                if current_parent == house_name {
                    display_parts.push("Current");
                }
                if selected_parent == house_name {
                    display_parts.push("Selected");
                }

                let display_name = if display_parts.is_empty() {
                    item_name.clone()
                } else {
                    format!("{} ({})", item_name, display_parts.join(", "))
                };

                let color = if inspected_item
                    .as_ref()
                    .is_some_and(|name| name == item_name)
                {
                    YELLOW
                } else if current_parent == house_name || selected_parent == house_name {
                    colorix().accent().text_1()
                } else {
                    colorix().high_contrast_text()
                };

                display_name.cstr_c(color).label(ui)
            })
            .with_hover(|item_name, _ctx, ui| {
                let (_, selected_parent) = item_parents
                    .get(item_name)
                    .map(|(top, player)| (top.as_str(), player.as_str()))
                    .unwrap_or(("", ""));

                ui.horizontal(|ui| {
                    if ui.button("Inspect").clicked() {
                        pending_actions.push(inspect_action(item_name.clone()));
                    }
                    if selected_parent != house_name {
                        if ui.button("Select").clicked() {
                            if let Some(house_node) = cache.houses.get(house_name) {
                                if is_ability {
                                    if let Some(item_node) = cache.abilities.get(item_name) {
                                        cn().reducers
                                            .content_select_link(
                                                house_node.0.id(),
                                                item_node.0.id(),
                                            )
                                            .notify_op();
                                    }
                                } else {
                                    if let Some(item_node) = cache.statuses.get(item_name) {
                                        cn().reducers
                                            .content_select_link(
                                                house_node.0.id(),
                                                item_node.0.id(),
                                            )
                                            .notify_op();
                                    }
                                }
                            }
                        }
                    }
                });
            })
            .compose(&EMPTY_CONTEXT, ui);

        Ok(())
    }

    fn render_component_list(
        ui: &mut Ui,
        kind: NodeKind,
        parent_id: u64,
        current_component_id: Option<u64>,
        cache: &ExplorerCache,
    ) -> NodeResult<()> {
        node_kind_match!(
            kind,
            if "➕ Add New".cstr().button(ui).clicked() {
                op(|world| {
                    NodeType::default().open_publish_window(world);
                });
            }
        );
        if let Some(current_id) = current_component_id {
            let ctx = &EMPTY_CONTEXT;
            node_kind_match!(kind, {
                if let Ok(node) = ctx.load::<NodeType>(current_id) {
                    node.as_display().compose(ctx, ui);
                    ui.separator();
                }
            });
        }

        if let Some(component_nodes) = cache.component_nodes.get(&kind) {
            component_nodes
                .as_list(|(node_id, node_name), _ctx, ui| {
                    let color = if current_component_id == Some(*node_id) {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    node_name.cstr_c(color).label(ui)
                })
                .with_hover(|(node_id, _node_name), _ctx, ui| {
                    if current_component_id != Some(*node_id) {
                        if ui.button("Select").clicked() {
                            cn().reducers
                                .content_select_link(parent_id, *node_id)
                                .notify_op();
                        }
                    }
                })
                .compose(&EMPTY_CONTEXT, ui);
        }

        Ok(())
    }

    pub fn pane_units_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_node_list(ui, world, NamedNodeKind::NUnit)
    }

    pub fn pane_houses_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_node_list(ui, world, NamedNodeKind::NHouse)
    }

    pub fn pane_abilities_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_node_list(ui, world, NamedNodeKind::NAbilityMagic)
    }

    pub fn pane_statuses_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_node_list(ui, world, NamedNodeKind::NStatusMagic)
    }

    pub fn pane_house_units_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        if "➕ Add new".cstr().button(ui).clicked() {
            NUnit::default().open_publish_window(world);
        }
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();
            let house_name = state.inspected_house.as_ref().to_not_found()?.clone();
            let inspected_unit = state.inspected_unit.clone();
            cache
                .unit_names
                .as_list(|unit_name, _ctx, ui| {
                    let (current_parent, selected_parent) = cache
                        .unit_parents
                        .get(unit_name.as_str())
                        .map(|(top, player)| (top.as_str(), player.as_str()))
                        .unwrap_or(("", ""));

                    let mut display_parts = vec![];
                    if current_parent == house_name {
                        display_parts.push("Current");
                    }
                    if selected_parent == house_name {
                        display_parts.push("Selected");
                    }

                    let display_name = if display_parts.is_empty() {
                        unit_name.clone()
                    } else {
                        format!("{} ({})", unit_name, display_parts.join(", "))
                    };

                    let color = if inspected_unit
                        .as_ref()
                        .is_some_and(|name| name == unit_name)
                    {
                        YELLOW
                    } else if current_parent == house_name || selected_parent == house_name {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    display_name.cstr_c(color).label(ui)
                })
                .with_hover(|unit_name, _ctx, ui| {
                    let (_, selected_parent) = cache
                        .unit_parents
                        .get(unit_name.as_str())
                        .map(|(top, player)| (top.as_str(), player.as_str()))
                        .unwrap_or(("", ""));

                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            state
                                .pending_actions
                                .push(ExplorerAction::InspectUnit(unit_name.clone()));
                        }
                        if selected_parent != house_name {
                            if ui.button("Select").clicked() {
                                if let (Some(house_node), Some(unit_node)) = (
                                    cache.houses.get(house_name.as_str()),
                                    cache.units.get(unit_name.as_str()),
                                ) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), unit_node.0.id())
                                        .notify_op();
                                }
                            }
                        }
                    });
                })
                .compose(&EMPTY_CONTEXT, ui);

            Ok(())
        })
    }

    pub fn pane_house_abilities_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        if "➕ Add new".cstr().button(ui).clicked() {
            NAbilityMagic::default().open_publish_window(world);
        }
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();
            let house_name = state.inspected_house.as_ref().to_not_found()?.clone();
            let inspected_ability = state.inspected_ability.clone();

            Self::render_house_child_list_single(
                ui,
                &house_name,
                &cache.ability_names,
                &cache.ability_parents,
                &inspected_ability,
                ExplorerAction::InspectAbility,
                true,
                cache,
                &mut state.pending_actions,
            )
        })
    }

    pub fn pane_house_statuses_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        if "➕ Add new".cstr().button(ui).clicked() {
            NStatusMagic::default().open_publish_window(world);
        }
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();
            let house_name = state.inspected_house.as_ref().to_not_found()?.clone();
            let inspected_status = state.inspected_status.clone();

            Self::render_house_child_list_single(
                ui,
                &house_name,
                &cache.status_names,
                &cache.status_parents,
                &inspected_status,
                ExplorerAction::InspectStatus,
                false,
                cache,
                &mut state.pending_actions,
            )
        })
    }

    pub fn pane_unit_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        if "➕ Add new".cstr().button(ui).clicked() {
            NHouse::default().open_publish_window(world);
        }
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();

            let unit_name = state.inspected_unit.as_ref().to_not_found()?.clone();
            let parents = cache.unit_parents.get(&unit_name);
            let unit_node = cache.units.get(&unit_name).to_not_found()?.0.clone();
            let (current_parent, selected_parent) = parents
                .map(|(top, player)| (top.as_str(), player.as_str()))
                .unwrap_or(("", ""));
            let inspected_house = state.inspected_house.clone();

            Self::render_parent_link_list(
                ui,
                &cache.house_names,
                current_parent,
                selected_parent,
                &inspected_house,
                unit_node.id(),
                ExplorerAction::InspectHouse,
                cache,
                &mut state.pending_actions,
            )
        })
    }

    pub fn pane_ability_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        if "➕ Add new".cstr().button(ui).clicked() {
            NHouse::default().open_publish_window(world);
        }
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();

            let ability_name = state.inspected_ability.as_ref().to_not_found()?.clone();
            let parents = cache.ability_parents.get(&ability_name);
            let ability_node = cache.abilities.get(&ability_name).to_not_found()?.0.clone();
            let (current_parent, selected_parent) = parents
                .map(|(top, player)| (top.as_str(), player.as_str()))
                .unwrap_or(("", ""));
            let inspected_house = state.inspected_house.clone();

            Self::render_parent_link_list(
                ui,
                &cache.house_names,
                current_parent,
                selected_parent,
                &inspected_house,
                ability_node.id(),
                ExplorerAction::InspectHouse,
                cache,
                &mut state.pending_actions,
            )
        })
    }

    pub fn pane_status_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        if "➕ Add new".cstr().button(ui).clicked() {
            NHouse::default().open_publish_window(world);
        }
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let cache = world.resource::<ExplorerCache>();

            let status_name = state.inspected_status.as_ref().to_not_found()?.clone();
            let parents = cache.status_parents.get(&status_name);
            let status_node = cache.statuses.get(&status_name).to_not_found()?.0.clone();
            let (current_parent, selected_parent) = parents
                .map(|(top, player)| (top.as_str(), player.as_str()))
                .unwrap_or(("", ""));
            let inspected_house = state.inspected_house.clone();

            Self::render_parent_link_list(
                ui,
                &cache.house_names,
                current_parent,
                selected_parent,
                &inspected_house,
                status_node.id(),
                ExplorerAction::InspectHouse,
                cache,
                &mut state.pending_actions,
            )
        })
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let unit = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let unit_name = state.inspected_unit.as_ref().to_not_found()?.clone();
            cache.units.get(&unit_name).to_not_found()?.1.clone()
        };

        Self::render_node_card(&unit, ui)
    }

    pub fn pane_house_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let house = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let house_name = state.inspected_house.as_ref().to_not_found()?.clone();
            cache.houses.get(&house_name).to_not_found()?.1.clone()
        };

        Self::render_node_card(&house, ui)
    }

    pub fn pane_ability_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let ability = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let ability_name = state.inspected_ability.as_ref().to_not_found()?.clone();
            cache.abilities.get(&ability_name).to_not_found()?.1.clone()
        };

        Self::render_node_card(&ability, ui)
    }

    pub fn pane_status_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let status = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let status_name = state.inspected_status.as_ref().to_not_found()?.clone();
            cache.statuses.get(&status_name).to_not_found()?.1.clone()
        };

        Self::render_node_card(&status, ui)
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (parent, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let unit_name = state.inspected_unit.as_ref().to_not_found()?.clone();
            let unit = &cache.units.get(&unit_name).to_not_found()?.1;
            let current_id = unit.description().ok().map(|desc| desc.id);
            (unit.id(), current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NUnitDescription,
            parent,
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (description_id, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let unit_name = state.inspected_unit.as_ref().to_not_found()?;
            let unit = &cache.units.get(unit_name).to_not_found()?.1;
            let description = unit.description()?;
            let current_id = description.behavior().ok().map(|behavior| behavior.id);
            (description.id, current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NUnitBehavior,
            description_id,
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (parent, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let unit_name = state.inspected_unit.as_ref().to_not_found()?;
            let unit = &cache.units.get(unit_name).to_not_found()?.1;
            let current_id = unit.stats().ok().map(|stats| stats.id);
            (unit.id(), current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NUnitStats,
            parent,
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_unit_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (description_id, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let unit_name = state.inspected_unit.as_ref().to_not_found()?;
            let unit = &cache.units.get(unit_name).to_not_found()?.1;
            let description = unit.description()?;
            let current_id = description
                .representation()
                .ok()
                .map(|representation| representation.id);
            (description.id, current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NUnitRepresentation,
            description_id,
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (house, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let house_name = state.inspected_house.as_ref().to_not_found()?;
            let house = &cache.houses.get(house_name).to_not_found()?.1;
            let current_id = house.color().ok().map(|color| color.id);
            (house.clone(), current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NHouseColor,
            house.id(),
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_ability_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (ability, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let ability_name = state.inspected_ability.as_ref().to_not_found()?.clone();
            let ability = cache.abilities.get(&ability_name).to_not_found()?.1.clone();
            let current_id = ability.description().ok().map(|desc| desc.id);
            (ability.clone(), current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NAbilityDescription,
            ability.id(),
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_ability_effect(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (description_id, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let ability_name = state.inspected_ability.as_ref().to_not_found()?.clone();
            let ability = cache.abilities.get(&ability_name).to_not_found()?.1.clone();
            let description = ability.description()?;
            let current_id = description.effect().ok().map(|effect| effect.id);
            (description.id, current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NAbilityEffect,
            description_id,
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_status_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (status, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let status_name = state.inspected_status.as_ref().to_not_found()?.clone();
            let status = cache.statuses.get(&status_name).to_not_found()?.1.clone();
            let current_id = status.description().ok().map(|desc| desc.id);
            (status.clone(), current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NStatusDescription,
            status.id(),
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (description_id, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let status_name = state.inspected_status.as_ref().to_not_found()?.clone();
            let status = cache.statuses.get(&status_name).to_not_found()?.1.clone();
            let description = status.description()?;
            let current_id = description.behavior().ok().map(|behavior| behavior.id);
            (description.id, current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NStatusBehavior,
            description_id,
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }

    pub fn pane_status_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let (status, current_component_id) = {
            let cache = world.resource::<ExplorerCache>();
            let state = world.resource::<ExplorerState>();
            let status_name = state.inspected_status.as_ref().to_not_found()?.clone();
            let status = cache.statuses.get(&status_name).to_not_found()?.1.clone();
            let current_id = status
                .representation()
                .ok()
                .map(|representation| representation.id);
            (status, current_id)
        };

        Self::render_component_list(
            ui,
            NodeKind::NStatusRepresentation,
            status.id(),
            current_component_id,
            &world.resource::<ExplorerCache>(),
        )
    }
}
