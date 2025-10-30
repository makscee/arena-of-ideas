use super::*;

pub struct ExplorerPanes;

impl ExplorerPanes {
    fn show_add_new(ui: &mut Ui, kind: NodeKind) {
        node_kind_match!(
            kind,
            if "➕ Add new".cstr().button(ui).clicked() {
                op(|world| {
                    NodeType::default().open_publish_window(world);
                });
            }
        );
    }
    pub fn render_node_list(ui: &mut Ui, world: &mut World, kind: NamedNodeKind) -> NodeResult<()> {
        Self::show_add_new(ui, kind.to_kind());

        world.resource_scope::<ExplorerState, _>(|_, mut state| {
            state.view_mode.exec_ctx(|ctx| {
                let mut node_list = named_node_kind_match!(kind, {
                    ctx.world_mut()?
                        .query::<(Entity, &NamedNodeType)>()
                        .iter(ctx.world()?)
                        .map(|(_, n)| (n.id, n.name().to_owned()))
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
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            if let Some(house_id) = state.inspected_house {
                Self::render_house_selection_list::<NUnit>(ui, &mut state, house_id, true)
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_abilities_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            if let Some(house_id) = state.inspected_house {
                Self::render_house_selection_list::<NAbilityMagic>(ui, &mut state, house_id, false)
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_statuses_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            if let Some(house_id) = state.inspected_house {
                Self::render_house_selection_list::<NStatusMagic>(ui, &mut state, house_id, false)
            } else {
                Ok(())
            }
        })
    }

    fn pane_component<T: ClientNode + FDisplay + FDescription>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        parent: u64,
    ) -> NodeResult<()> {
        let kind = T::kind_s();
        Self::show_add_new(ui, kind);
        let mut current_id = None;
        ctx.exec_mut(|ctx| {
            let child = ctx
                .get_children_of_kind(parent, kind)?
                .into_iter()
                .next()
                .to_custom_err_fn(|| format!("Child {kind} of {parent} not found"))?;
            let node = ctx.load::<T>(child)?;
            format!("[s [tw {}]]", node.id()).label(ui);
            current_id = Some(node.id());
            node.display(ctx, ui);
            Ok(())
        })
        .ui(ui);
        ctx.world_mut()?
            .query::<&T>()
            .iter(ctx.world()?)
            .collect_vec()
            .as_list(|node, _ctx, ui| {
                let is_current = current_id == Some(node.id());
                let text = node.description_cstr(ctx);
                let text = if is_current {
                    format!("⚫️ {text}")
                } else {
                    text
                };
                text.cstr().label(ui)
            })
            .with_hover(move |node, _, ui| {
                if "Select".cstr().button(ui).clicked() {
                    cn().reducers
                        .content_select_link(parent, node.id())
                        .notify_error_op();
                }
            })
            .compose(ctx, ui);
        Ok(())
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    let parent = ctx.load::<NUnit>(unit_id)?.id;
                    Self::pane_component::<NUnitDescription>(ui, ctx, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            if let Some(unit_id) = state.inspected_unit {
                Self::render_parent_list(ui, &mut state, unit_id, NodeKind::NHouse)
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                state.view_mode.exec_ctx(|ctx| {
                    let parent = ctx.load::<NUnit>(unit_id)?.description_ref(ctx)?.id;
                    Self::pane_component::<NUnitBehavior>(ui, ctx, parent)
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
                    let parent = ctx.load::<NUnit>(unit_id)?.id;
                    Self::pane_component::<NUnitStats>(ui, ctx, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            if let Some(ability_id) = state.inspected_ability {
                Self::render_parent_list(ui, &mut state, ability_id, NodeKind::NHouse)
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_parent_list(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, mut state| {
            if let Some(status_id) = state.inspected_status {
                Self::render_parent_list(ui, &mut state, status_id, NodeKind::NHouse)
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
                    Self::render_node_card(ctx.load_ref::<NUnit>(unit_id)?, ctx, ui)
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
                    Self::render_node_card(ctx.load_ref::<NHouse>(house_id)?, ctx, ui)
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
                    Self::render_node_card(ctx.load_ref::<NAbilityMagic>(ability_id)?, ctx, ui)
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
                    Self::render_node_card(ctx.load_ref::<NStatusMagic>(status_id)?, ctx, ui)
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
                    let parent = ctx.load::<NUnit>(unit_id)?.description_ref(ctx)?.id;
                    Self::pane_component::<NUnitRepresentation>(ui, ctx, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                state
                    .view_mode
                    .exec_ctx(|ctx| Self::pane_component::<NHouseColor>(ui, ctx, house_id))
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(ability_id) = state.inspected_ability {
                state.view_mode.exec_ctx(|ctx| {
                    Self::pane_component::<NAbilityDescription>(ui, ctx, ability_id)
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
                    let parent = ctx
                        .load::<NAbilityMagic>(ability_id)?
                        .description_ref(ctx)?
                        .id;
                    Self::pane_component::<NAbilityEffect>(ui, ctx, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state
                    .view_mode
                    .exec_ctx(|ctx| Self::pane_component::<NStatusDescription>(ui, ctx, status_id))
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(status_id) = state.inspected_status {
                state.view_mode.exec_ctx(|ctx| {
                    let parent = ctx
                        .load::<NStatusMagic>(status_id)?
                        .description_ref(ctx)?
                        .id;
                    Self::pane_component::<NStatusBehavior>(ui, ctx, parent)
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
                    Self::pane_component::<NStatusRepresentation>(ui, ctx, status_id)
                })
            } else {
                Ok(())
            }
        })
    }

    fn render_parent_list(
        ui: &mut Ui,
        state: &mut ExplorerState,
        node_id: u64,
        parent_kind: NodeKind,
    ) -> NodeResult<()> {
        Self::show_add_new(ui, parent_kind);
        state.view_mode.exec_ctx(|ctx| {
            let current_parent = ctx
                .get_parents_of_kind(node_id, parent_kind)?
                .into_iter()
                .next();

            let mut all_nodes = Vec::new();
            match parent_kind {
                NodeKind::NHouse => {
                    for house in ctx.world_mut()?.query::<&NHouse>().iter(ctx.world()?) {
                        all_nodes.push((house.id, house.name().to_string()));
                    }
                }
                _ => return Ok(()),
            }

            all_nodes.sort_by_key(|(id, _)| *id);

            all_nodes
                .as_list(|(id, name), _ctx, ui| {
                    let is_current = current_parent == Some(*id);
                    let text = if is_current {
                        format!("● {}", name)
                    } else {
                        name.clone()
                    };
                    text.cstr().label(ui)
                })
                .with_hover(move |(_id, _name), _, ui| {
                    if "Select".cstr().button(ui).clicked() {
                        // Parent selection logic would go here if needed
                    }
                })
                .compose(ctx, ui);

            Ok(())
        })
    }

    fn render_house_selection_list<T>(
        ui: &mut Ui,
        state: &mut ExplorerState,
        house_id: u64,
        multiple_selection: bool,
    ) -> NodeResult<()>
    where
        T: ClientNode + NamedNode,
    {
        Self::show_add_new(ui, NodeKind::NHouse);
        state.view_mode.exec_ctx(|ctx| {
            let current_children = if multiple_selection {
                ctx.get_children_of_kind(house_id, T::named_kind().to_kind())?
                    .into_iter()
                    .collect::<HashSet<_>>()
            } else {
                ctx.get_children_of_kind(house_id, T::named_kind().to_kind())?
                    .into_iter()
                    .take(1)
                    .collect::<HashSet<_>>()
            };

            let mut all_nodes = Vec::new();
            for node in ctx.world_mut()?.query::<&T>().iter(ctx.world()?) {
                all_nodes.push((node.id(), node.name().to_string()));
            }

            all_nodes.sort_by_key(|(id, _)| *id);

            all_nodes
                .as_list(|(id, name), _ctx, ui| {
                    let is_current = current_children.contains(id);
                    let text = if is_current {
                        format!("● {}", name)
                    } else {
                        name.clone()
                    };
                    text.cstr().label(ui)
                })
                .with_hover(move |(node_id, _name), _, ui| {
                    if "Select".cstr().button(ui).clicked() {
                        cn().reducers
                            .content_select_link(house_id, *node_id)
                            .notify_error_op();
                    }
                })
                .compose(ctx, ui);

            Ok(())
        })
    }
}
