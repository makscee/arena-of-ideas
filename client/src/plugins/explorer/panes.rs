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
            with_core_source(|ctx| {
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
                    NamedNodeKind::NAbilityMagic => None,
                    NamedNodeKind::NStatusMagic => None,
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
                                NamedNodeKind::NAbilityMagic => return,
                                NamedNodeKind::NStatusMagic => return,
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
        format!("[s [tw {}]]", node.id())
            .as_label(ui.style())
            .selectable(true)
            .ui(ui);
        node.as_card().compose(ctx, ui);
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

    pub fn pane_house_abilities_list(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        ui.label("House abilities will be shown here");
        Ok(())
    }

    pub fn pane_house_statuses_list(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        ui.label("House statuses will be shown here");
        Ok(())
    }

    fn pane_component<T: ClientNode + FDisplay + FDescription, P: ClientNode>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        owner: u64,
        parent: P,
        filter: Option<fn(&T, &P) -> bool>,
    ) -> NodeResult<()> {
        let kind = T::kind_s();
        Self::show_add_new(ui, kind);
        let mut current_id = None;
        let parent_id = parent.id();
        ctx.exec_mut(|ctx| {
            let child = ctx
                .get_children_of_kind(parent_id, kind)?
                .into_iter()
                .next()
                .to_custom_err_fn(|| format!("Child {kind} of {parent_id} not found"))?;
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
            .filter(|n| {
                if let Some(filter) = filter {
                    filter(*n, &parent)
                } else {
                    true
                }
            })
            .unique_by(|n| n.id())
            .collect_vec()
            .as_list(|node, ctx, ui| {
                let is_current = current_id == Some(node.id());
                let text = ctx
                    .exec_ref(|ctx| {
                        ctx.set_owner(owner);
                        Ok(node.description_cstr(ctx))
                    })
                    .unwrap();
                let text = if is_current {
                    format!("* {text}")
                } else {
                    text
                };
                text.cstr().label_w(ui)
            })
            .with_hover(move |node, _, ui| {
                if "Select".cstr().button(ui).clicked() {
                    ui.horizontal(|ui| {
                        if "⬆".cstr().button(ui).clicked() {
                            cn().reducers
                                .content_upvote_node(node.id())
                                .notify_error_op();
                        }
                        if "⬇".cstr().button(ui).clicked() {
                            cn().reducers
                                .content_downvote_node(node.id())
                                .notify_error_op();
                        }
                        ui.label(format!("Rating: {}", node.rating()));
                    });
                }
            })
            .compose(ctx, ui);
        Ok(())
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_core_source(|ctx| {
                    let parent = ctx.load::<NUnit>(unit_id)?;
                    Self::pane_component::<NUnitDescription, _>(ui, ctx, unit_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_parent_list(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        ui.label("Unit parent house will be shown here");
        Ok(())
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_core_source(|ctx| {
                    let parent = ctx.load::<NUnit>(unit_id)?.description_ref(ctx)?.clone();
                    Self::pane_component::<NUnitBehavior, _>(
                        ui,
                        ctx,
                        unit_id,
                        parent,
                        Some(|n, p| {
                            n.magic_type == p.magic_type && n.reaction.trigger == p.trigger
                        }),
                    )
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_core_source(|ctx| {
                    let parent = ctx.load::<NUnit>(unit_id)?;
                    Self::pane_component::<NUnitStats, _>(ui, ctx, unit_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_parent_list(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        ui.label("Ability parent house will be shown here");
        Ok(())
    }

    pub fn pane_status_parent_list(ui: &mut Ui, _world: &mut World) -> NodeResult<()> {
        ui.label("Status parent house will be shown here");
        Ok(())
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_core_source(|ctx| {
                    Self::render_node_card(ctx.load_ref::<NUnit>(unit_id)?, ctx, ui)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_core_source(|ctx| {
                    Self::render_node_card(ctx.load_ref::<NHouse>(house_id)?, ctx, ui)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_representation(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_core_source(|ctx| {
                    let parent = ctx.load::<NUnit>(unit_id)?.description_ref(ctx)?.clone();
                    Self::pane_component::<NUnitRepresentation, _>(ui, ctx, unit_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_core_source(|ctx| {
                    Self::pane_component::<NHouseColor, _>(
                        ui,
                        ctx,
                        house_id,
                        ctx.load::<NHouse>(house_id)?,
                        None,
                    )
                })
            } else {
                Ok(())
            }
        })
    }
}
