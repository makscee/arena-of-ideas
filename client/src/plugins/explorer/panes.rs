use super::*;

pub struct ExplorerPanes;

impl ExplorerPanes {
    pub fn render_node_list(ui: &mut Ui, world: &mut World, kind: NamedNodeKind) -> NodeResult<()> {
        let mut state = world.remove_resource::<ExplorerState>().to_not_found()?;

        let (names, inspected_name): (Vec<&String>, Option<String>) = match kind {
            NamedNodeKind::NUnit => {
                let names: Vec<&String> = state.cache.units.keys().collect();
                (names, state.inspected_unit.clone())
            }
            NamedNodeKind::NHouse => {
                let names: Vec<&String> = state.cache.houses.keys().collect();
                (names, state.inspected_house.clone())
            }
            NamedNodeKind::NAbilityMagic => {
                let names: Vec<&String> = state.cache.abilities.keys().collect();
                (names, state.inspected_ability.clone())
            }
            NamedNodeKind::NStatusMagic => {
                let names: Vec<&String> = state.cache.statuses.keys().collect();
                (names, state.inspected_status.clone())
            }
        };

        names
            .as_list(|name, _ctx, ui| {
                let color = if inspected_name
                    .as_ref()
                    .is_some_and(|inspected| inspected.eq(*name))
                {
                    YELLOW
                } else {
                    colorix().high_contrast_text()
                };
                name.cstr_c(color).label(ui)
            })
            .with_hover(|name, _ctx, ui| {
                if ui.button("Inspect").clicked() {
                    let action = match kind {
                        NamedNodeKind::NUnit => ExplorerAction::InspectUnit(name.to_string()),
                        NamedNodeKind::NHouse => ExplorerAction::InspectHouse(name.to_string()),
                        NamedNodeKind::NAbilityMagic => {
                            ExplorerAction::InspectAbility(name.to_string())
                        }
                        NamedNodeKind::NStatusMagic => {
                            ExplorerAction::InspectStatus(name.to_string())
                        }
                    };

                    state.pending_actions.push(action);
                }
            })
            .compose(&cn().db().as_context(), ui);
        world.insert_resource(state);

        Ok(())
    }

    pub fn render_node_card<T>(node: &T, ui: &mut Ui) -> NodeResult<()>
    where
        T: ClientNode + FCard,
    {
        node.as_card().compose(&cn().db.as_context(), ui);
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
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let ExplorerState {
                inspected_unit,
                inspected_house,
                inspected_ability: _,
                inspected_status: _,
                view_mode: _,
                cache,
                pending_actions,
            } = state.as_mut();
            let house_name = inspected_house.as_ref().to_not_found()?.clone();
            let inspected_unit = inspected_unit.clone();

            cache
                .unit_names
                .as_list(|unit_name, _ctx, ui| {
                    let (current_parent, selected_parent) = cache
                        .unit_parents
                        .get(unit_name.as_str())
                        .map(|(top, player)| (top.clone(), player.clone()))
                        .unwrap_or_default();

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
                        .map(|(top, player)| (top.clone(), player.clone()))
                        .unwrap_or_default();

                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            pending_actions.push(ExplorerAction::InspectUnit(unit_name.clone()));
                        }
                        if selected_parent != house_name {
                            if ui.button("Select").clicked() {
                                let state = world.resource::<ExplorerState>();
                                if let (Some(house_node), Some(unit_node)) = (
                                    state.cache.houses.get(house_name.as_str()),
                                    state.cache.units.get(unit_name.as_str()),
                                ) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), unit_node.0.id())
                                        .notify(world);
                                }
                            }
                        }
                    });
                })
                .compose(&cn().db.as_context(), ui);

            Ok(())
        })
    }

    pub fn pane_house_abilities_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let ExplorerState {
                inspected_unit: _,
                inspected_house,
                inspected_ability,
                inspected_status: _,
                view_mode: _,
                cache,
                pending_actions,
            } = state.as_mut();
            let house_name = inspected_house.as_ref().to_not_found()?.clone();
            let inspected_ability = inspected_ability.clone();

            cache
                .ability_names
                .as_list(|ability_name, _ctx, ui| {
                    let (current_parent, selected_parent) = cache
                        .ability_parents
                        .get(ability_name.as_str())
                        .map(|(top, player)| (top.clone(), player.clone()))
                        .unwrap_or_default();

                    let mut display_parts = vec![];
                    if current_parent == house_name {
                        display_parts.push("Current");
                    }
                    if selected_parent == house_name {
                        display_parts.push("Selected");
                    }

                    let display_name = if display_parts.is_empty() {
                        ability_name.clone()
                    } else {
                        format!("{} ({})", ability_name, display_parts.join(", "))
                    };

                    let color = if inspected_ability
                        .as_ref()
                        .is_some_and(|name| name == ability_name)
                    {
                        YELLOW
                    } else if current_parent == house_name || selected_parent == house_name {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    display_name.cstr_c(color).label(ui)
                })
                .with_hover(|ability_name, _ctx, ui| {
                    let (_, selected_parent) = cache
                        .ability_parents
                        .get(ability_name.as_str())
                        .map(|(top, player)| (top.clone(), player.clone()))
                        .unwrap_or_default();

                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            pending_actions
                                .push(ExplorerAction::InspectAbility(ability_name.clone()));
                        }
                        if selected_parent != house_name {
                            if ui.button("Select").clicked() {
                                let state = world.resource::<ExplorerState>();
                                if let (Some(house_node), Some(ability_node)) = (
                                    state.cache.houses.get(house_name.as_str()),
                                    state.cache.abilities.get(ability_name.as_str()),
                                ) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), ability_node.0.id())
                                        .notify(world);
                                }
                            }
                        }
                    });
                })
                .compose(&cn().db.as_context(), ui);

            Ok(())
        })
    }

    pub fn pane_house_statuses_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let ExplorerState {
                inspected_unit: _,
                inspected_house,
                inspected_ability: _,
                inspected_status,
                view_mode: _,
                cache,
                pending_actions,
            } = state.as_mut();
            let house_name = inspected_house.as_ref().to_not_found()?.clone();
            let inspected_status = inspected_status.clone();

            cache
                .status_names
                .as_list(|status_name, _ctx, ui| {
                    let (current_parent, selected_parent) = cache
                        .status_parents
                        .get(status_name.as_str())
                        .map(|(top, player)| (top.clone(), player.clone()))
                        .unwrap_or_default();

                    let mut display_parts = vec![];
                    if current_parent == house_name {
                        display_parts.push("Current");
                    }
                    if selected_parent == house_name {
                        display_parts.push("Selected");
                    }

                    let display_name = if display_parts.is_empty() {
                        status_name.clone()
                    } else {
                        format!("{} ({})", status_name, display_parts.join(", "))
                    };

                    let color = if inspected_status
                        .as_ref()
                        .is_some_and(|name| name == status_name)
                    {
                        YELLOW
                    } else if current_parent == house_name || selected_parent == house_name {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    display_name.cstr_c(color).label(ui)
                })
                .with_hover(|status_name, _ctx, ui| {
                    let (_, selected_parent) = cache
                        .status_parents
                        .get(status_name.as_str())
                        .map(|(top, player)| (top.clone(), player.clone()))
                        .unwrap_or_default();

                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            pending_actions
                                .push(ExplorerAction::InspectStatus(status_name.clone()));
                        }
                        if selected_parent != house_name {
                            if ui.button("Select").clicked() {
                                if let (Some(house_node), Some(status_node)) = (
                                    cache.houses.get(house_name.as_str()),
                                    cache.statuses.get(status_name.as_str()),
                                ) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), status_node.0.id())
                                        .notify(world);
                                }
                            }
                        }
                    });
                })
                .compose(&cn().db.as_context(), ui);

            Ok(())
        })
    }

    pub fn pane_unit_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let ExplorerState {
                inspected_unit,
                inspected_house,
                inspected_ability: _,
                inspected_status: _,
                view_mode: _,
                cache,
                pending_actions,
            } = state.as_mut();
            let unit_name = inspected_unit.as_ref().to_not_found()?.clone();
            let parents = cache.unit_parents.get(unit_name.as_str()).cloned();
            let inspected_house = inspected_house.clone();
            let unit_node = cache
                .units
                .get(unit_name.as_str())
                .to_not_found()?
                .0
                .clone();

            let (current_parent, selected_parent) = parents
                .map(|(top, player)| (top.clone(), player.clone()))
                .unwrap_or_default();

            cache
                .house_names
                .as_list(|house_name, _ctx, ui| {
                    let mut display_parts = vec![];
                    if house_name == &current_parent {
                        display_parts.push("Current");
                    }
                    if house_name == &selected_parent {
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
                    } else if house_name == &current_parent || house_name == &selected_parent {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    display_name.cstr_c(color).label(ui)
                })
                .with_hover(|house_name, _ctx, ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            pending_actions.push(ExplorerAction::InspectHouse(house_name.clone()));
                        }
                        if house_name != &selected_parent {
                            if ui.button("Select").clicked() {
                                if let Some(house_node) = cache.houses.get(house_name.as_str()) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), unit_node.id())
                                        .notify(world);
                                }
                            }
                        }
                    });
                })
                .compose(&cn().db.as_context(), ui);

            Ok(())
        })
    }

    pub fn pane_ability_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let ExplorerState {
                inspected_unit: _,
                inspected_house,
                inspected_ability,
                inspected_status: _,
                view_mode: _,
                cache,
                pending_actions,
            } = state.as_mut();
            let ability_name = inspected_ability.as_ref().to_not_found()?.clone();
            let parents = cache.ability_parents.get(ability_name.as_str()).cloned();
            let inspected_house = inspected_house.clone();
            let ability_node = cache
                .abilities
                .get(ability_name.as_str())
                .to_not_found()?
                .0
                .clone();

            let (current_parent, selected_parent) = parents
                .map(|(top, player)| (top.clone(), player.clone()))
                .unwrap_or_default();

            cache
                .house_names
                .as_list(|house_name, _ctx, ui| {
                    let mut display_parts = vec![];
                    if house_name == &current_parent {
                        display_parts.push("Current");
                    }
                    if house_name == &selected_parent {
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
                    } else if house_name == &current_parent || house_name == &selected_parent {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    display_name.cstr_c(color).label(ui)
                })
                .with_hover(|house_name, _ctx, ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            pending_actions.push(ExplorerAction::InspectHouse(house_name.clone()));
                        }
                        if house_name != &selected_parent {
                            if ui.button("Select").clicked() {
                                if let Some(house_node) = cache.houses.get(house_name.as_str()) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), ability_node.id())
                                        .notify(world);
                                }
                            }
                        }
                    });
                })
                .compose(&cn().db.as_context(), ui);

            Ok(())
        })
    }

    pub fn pane_status_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|world, mut state| {
            let ExplorerState {
                inspected_unit: _,
                inspected_house,
                inspected_ability: _,
                inspected_status,
                view_mode: _,
                cache,
                pending_actions,
            } = state.as_mut();
            let status_name = inspected_status.as_ref().to_not_found()?.clone();
            let parents = cache.status_parents.get(status_name.as_str()).cloned();
            let inspected_house = inspected_house.clone();
            let status_node = cache
                .statuses
                .get(status_name.as_str())
                .to_not_found()?
                .0
                .clone();

            let (current_parent, selected_parent) = parents
                .map(|(top, player)| (top.clone(), player.clone()))
                .unwrap_or_default();
            cache
                .house_names
                .as_list(|house_name, _ctx, ui| {
                    let mut display_parts = vec![];
                    if house_name == &current_parent {
                        display_parts.push("Current");
                    }
                    if house_name == &selected_parent {
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
                    } else if house_name == &current_parent || house_name == &selected_parent {
                        colorix().accent().text_1()
                    } else {
                        colorix().high_contrast_text()
                    };

                    display_name.cstr_c(color).label(ui)
                })
                .with_hover(|house_name, _ctx, ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Inspect").clicked() {
                            pending_actions.push(ExplorerAction::InspectHouse(house_name.clone()));
                        }
                        if house_name != &selected_parent {
                            if ui.button("Select").clicked() {
                                if let Some(house_node) = cache.houses.get(house_name.as_str()) {
                                    cn().reducers
                                        .content_select_link(house_node.0.id(), status_node.id())
                                        .notify(world);
                                }
                            }
                        }
                    });
                })
                .compose(&cn().db.as_context(), ui);

            Ok(())
        })
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let unit_name = state.inspected_unit.as_ref().to_not_found()?;
        if let Some((unit, _)) = state.cache.units.get(unit_name) {
            Self::render_node_card(unit, ui)?;
        }
        Ok(())
    }

    pub fn pane_house_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let house_name = state.inspected_house.as_ref().to_not_found()?;
        if let Some((house, _)) = state.cache.houses.get(house_name) {
            Self::render_node_card(house, ui)?;
        }
        Ok(())
    }

    pub fn pane_ability_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let ability_name = state.inspected_ability.as_ref().to_not_found()?;
        if let Some((ability, _)) = state.cache.abilities.get(ability_name) {
            Self::render_node_card(ability, ui)?;
        }
        Ok(())
    }

    pub fn pane_status_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let status_name = state.inspected_status.as_ref().to_not_found()?;
        if let Some((status, _)) = state.cache.statuses.get(status_name) {
            Self::render_node_card(status, ui)?;
        }
        Ok(())
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let unit_name = state.inspected_unit.as_ref().to_not_found()?;
        if let Some((unit, _)) = state.cache.units.get(unit_name) {
            let description = unit.description()?;
            description.display(&cn().db.as_context(), ui);
        }
        Ok(())
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let unit_name = state.inspected_unit.as_ref().to_not_found()?;
        if let Some((unit, _)) = state.cache.units.get(unit_name) {
            if let Ok(description) = unit.description_ref(&cn().db.as_context()) {
                if let Ok(behavior) = description.behavior_ref(&cn().db.as_context()) {
                    behavior.display(&cn().db.as_context(), ui);
                }
            }
        }
        Ok(())
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let unit_name = state.inspected_unit.as_ref().to_not_found()?;
        if let Some((unit, _)) = state.cache.units.get(unit_name) {
            let stats = unit.stats()?;
            stats.display(&cn().db.as_context(), ui);
        }
        Ok(())
    }

    pub fn pane_unit_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let unit_name = state.inspected_unit.as_ref().to_not_found()?;
        if let Some((unit, _)) = state.cache.units.get(unit_name) {
            if let Ok(description) = unit.description_ref(&cn().db.as_context()) {
                if let Ok(representation) = description.representation_ref(&cn().db.as_context()) {
                    representation.display(&cn().db.as_context(), ui);
                }
            }
        }
        Ok(())
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let house_name = state.inspected_house.as_ref().to_not_found()?;
        if let Some((house, _)) = state.cache.houses.get(house_name) {
            let color = house.color()?;
            color.display(&cn().db.as_context(), ui);
        }
        Ok(())
    }

    pub fn pane_ability_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let ability_name = state.inspected_ability.as_ref().to_not_found()?;
        if let Some((ability, _)) = state.cache.abilities.get(ability_name) {
            let description = ability.description()?;
            description.display(&cn().db.as_context(), ui);
        }
        Ok(())
    }

    pub fn pane_ability_effect(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let ability_name = state.inspected_ability.as_ref().to_not_found()?;
        if let Some((ability, _)) = state.cache.abilities.get(ability_name) {
            if let Ok(description) = ability.description_ref(&cn().db.as_context()) {
                if let Ok(effect) = description.effect_ref(&cn().db.as_context()) {
                    effect.display(&cn().db.as_context(), ui);
                }
            }
        }
        Ok(())
    }

    pub fn pane_status_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let status_name = state.inspected_status.as_ref().to_not_found()?;
        if let Some((status, _)) = state.cache.statuses.get(status_name) {
            let description = status.description()?;
            description.display(&cn().db.as_context(), ui);
        }
        Ok(())
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let status_name = state.inspected_status.as_ref().to_not_found()?;
        if let Some((status, _)) = state.cache.statuses.get(status_name) {
            if let Ok(description) = status.description_ref(&cn().db.as_context()) {
                if let Ok(behavior) = description.behavior_ref(&cn().db.as_context()) {
                    behavior.display(&cn().db.as_context(), ui);
                }
            }
        }
        Ok(())
    }

    pub fn pane_status_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        let state = world.resource::<ExplorerState>();
        let status_name = state.inspected_status.as_ref().to_not_found()?;
        if let Some((status, _)) = state.cache.statuses.get(status_name) {
            if let Ok(representation) = status.representation_ref(&cn().db.as_context()) {
                representation.display(&cn().db.as_context(), ui);
            }
        }
        Ok(())
    }
}
