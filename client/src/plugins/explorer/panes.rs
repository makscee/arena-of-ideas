use super::*;

pub struct ExplorerPanes;

impl ExplorerPanes {
    pub fn render_node_list(ui: &mut Ui, world: &mut World, kind: NamedNodeKind) -> NodeResult<()> {
        named_node_kind_match!(
            kind,
            if "➕ Add new".cstr().button(ui).clicked() {
                NamedNodeType::default().open_publish_window(world);
            }
        );

        world.resource_scope::<ExplorerState, _>(|_, mut state| {
            state.view_mode.exec_ctx(|ctx| {
                let mut node_list = named_node_kind_match!(kind, {
                    ctx.world_mut()?
                        .query::<(Entity, &NamedNodeType)>()
                        .iter(ctx.world()?)
                        .map(|(e, n)| (n.id, n.name().to_owned()))
                        .collect_vec()
                });
                node_list.sort_by_key(|(id, _)| *id);

                // Get inspected item
                let inspected_id = match kind {
                    NamedNodeKind::NUnit => state.inspected_unit,
                    NamedNodeKind::NHouse => state.inspected_house,
                    NamedNodeKind::NAbilityMagic => state.inspected_ability,
                    NamedNodeKind::NStatusMagic => state.inspected_status,
                };

                // Render the list
                node_list
                    .as_list(|(id, name), _ctx, ui| {
                        let color = if inspected_id == Some(*id) {
                            YELLOW
                        } else {
                            colorix().high_contrast_text()
                        };
                        name.cstr_c(color).label(ui)
                    })
                    .with_hover(|(id, _name), _ctx, ui| {
                        if ui.button("Inspect").clicked() {
                            let action = match kind {
                                NamedNodeKind::NUnit => ExplorerAction::InspectUnit(*id),
                                NamedNodeKind::NHouse => ExplorerAction::InspectHouse(*id),
                                NamedNodeKind::NAbilityMagic => ExplorerAction::InspectAbility(*id),
                                NamedNodeKind::NStatusMagic => ExplorerAction::InspectStatus(*id),
                            };

                            state.pending_actions.push(action);
                        }
                    })
                    .compose(ctx, ui);

                Ok(())
            })
        })
    }

    pub fn render_node_card<T>(node: &T, ctx: &ClientContext, ui: &mut Ui) -> NodeResult<()>
    where
        T: ClientNode + FCard,
    {
        node.as_card().compose(ctx, ui);
        Ok(())
    }

    pub fn render_view_mode_switch(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            ui.horizontal(|ui| {
                for mode in [ViewMode::Solid, ViewMode::Top, ViewMode::Selected] {
                    let is_current = state.view_mode == mode;
                    let color = if is_current {
                        YELLOW
                    } else {
                        colorix().high_contrast_text()
                    };

                    if mode.name().cstr_c(color).button(ui).clicked() {
                        state
                            .pending_actions
                            .push(ExplorerAction::SwitchViewMode(mode));
                    }
                }
            });
            Ok(())
        })
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
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                state.view_mode.exec_ctx(|ctx| {
                    let children = ctx.get_children_of_kind(house_id, NodeKind::NUnit)?;
                    let mut units = Vec::new();

                    for unit_id in children {
                        if let Ok(unit) = ctx.load_ref::<NUnit>(unit_id) {
                            units.push((unit_id, unit.name().to_string()));
                        }
                    }

                    units.sort_by_key(|(id, _)| *id);

                    units
                        .as_list(|(_id, name), _ctx, ui| name.cstr().label(ui))
                        .compose(ctx, ui);

                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_abilities_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                state.view_mode.exec_ctx(|ctx| {
                    let children = ctx.get_children_of_kind(house_id, NodeKind::NAbilityMagic)?;
                    let mut abilities = Vec::new();

                    for ability_id in children {
                        if let Ok(ability) = ctx.load_ref::<NAbilityMagic>(ability_id) {
                            abilities.push((ability_id, ability.name().to_string()));
                        }
                    }

                    abilities.sort_by_key(|(id, _)| *id);

                    abilities
                        .as_list(|(_id, name), _ctx, ui| name.cstr().label(ui))
                        .compose(ctx, ui);

                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_statuses_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                state.view_mode.exec_ctx(|ctx| {
                    let children = ctx.get_children_of_kind(house_id, NodeKind::NStatusMagic)?;
                    let mut statuses = Vec::new();

                    for status_id in children {
                        if let Ok(status) = ctx.load_ref::<NStatusMagic>(status_id) {
                            statuses.push((status_id, status.name().to_string()));
                        }
                    }

                    statuses.sort_by_key(|(id, _)| *id);

                    statuses
                        .as_list(|(_id, name), _ctx, ui| name.cstr().label(ui))
                        .compose(ctx, ui);

                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    let parents = ctx.get_parents_of_kind(unit_id, NodeKind::NHouse)?;
                    let mut houses = Vec::new();

                    for house_id in parents {
                        if let Ok(house) = ctx.load_ref::<NHouse>(house_id) {
                            houses.push((house_id, house.name().to_string()));
                        }
                    }

                    houses.sort_by_key(|(id, _)| *id);

                    houses
                        .as_list(|(_id, name), _ctx, ui| name.cstr().label(ui))
                        .compose(ctx, ui);

                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(ability_id) = state.inspected_ability {
                state.view_mode.exec_ctx(|ctx| {
                    let parents = ctx.get_parents_of_kind(ability_id, NodeKind::NHouse)?;
                    let mut houses = Vec::new();

                    for house_id in parents {
                        if let Ok(house) = ctx.load_ref::<NHouse>(house_id) {
                            houses.push((house_id, house.name().to_string()));
                        }
                    }

                    houses.sort_by_key(|(id, _)| *id);

                    houses
                        .as_list(|(_id, name), _ctx, ui| name.cstr().label(ui))
                        .compose(ctx, ui);

                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state.view_mode.exec_ctx(|ctx| {
                    let parents = ctx.get_parents_of_kind(status_id, NodeKind::NHouse)?;
                    let mut houses = Vec::new();

                    for house_id in parents {
                        if let Ok(house) = ctx.load_ref::<NHouse>(house_id) {
                            houses.push((house_id, house.name().to_string()));
                        }
                    }

                    houses.sort_by_key(|(id, _)| *id);

                    houses
                        .as_list(|(_id, name), _ctx, ui| name.cstr().label(ui))
                        .compose(ctx, ui);

                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_view_mode_switch(ui, world)?;

        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    if let Ok(unit) = ctx.load_ref::<NUnit>(unit_id) {
                        Self::render_node_card(&*unit, ctx, ui)?;
                    }
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_view_mode_switch(ui, world)?;

        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                state.view_mode.exec_ctx(|ctx| {
                    if let Ok(house) = ctx.load_ref::<NHouse>(house_id) {
                        Self::render_node_card(&*house, ctx, ui)?;
                    }
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_view_mode_switch(ui, world)?;

        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(ability_id) = state.inspected_ability {
                state.view_mode.exec_ctx(|ctx| {
                    if let Ok(ability) = ctx.load_ref::<NAbilityMagic>(ability_id) {
                        Self::render_node_card(&*ability, ctx, ui)?;
                    }
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        Self::render_view_mode_switch(ui, world)?;

        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state.view_mode.exec_ctx(|ctx| {
                    if let Ok(status) = ctx.load_ref::<NStatusMagic>(status_id) {
                        Self::render_node_card(&*status, ctx, ui)?;
                    }
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    if let Ok(unit) = ctx.load_ref::<NUnit>(unit_id) {
                        if let Ok(desc) = unit.description_ref(ctx) {
                            desc.display(ctx, ui);
                        }
                    }
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NUnit>(unit_id)?
                        .description_ref(ctx)?
                        .behavior_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NUnit>(unit_id)?
                        .stats_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NUnit>(unit_id)?
                        .description_ref(ctx)?
                        .representation_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NHouse>(house_id)?
                        .color_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(ability_id) = state.inspected_ability {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NAbilityMagic>(ability_id)?
                        .description_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_effect(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(ability_id) = state.inspected_ability {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NAbilityMagic>(ability_id)?
                        .description_ref(ctx)?
                        .effect_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NStatusMagic>(status_id)?
                        .description_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NStatusMagic>(status_id)?
                        .description_ref(ctx)?
                        .behavior_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state.view_mode.exec_ctx(|ctx| {
                    ctx.load_ref::<NStatusMagic>(status_id)?
                        .representation_ref(ctx)?
                        .display(ctx, ui);
                    Ok(())
                })
            } else {
                Ok(())
            }
        })
    }
}
