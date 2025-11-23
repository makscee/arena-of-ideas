use super::*;

pub struct ExplorerPanes;

impl ExplorerPanes {
    fn show_add_new(ui: &mut Ui, kind: NodeKind, parent: Option<u64>) {
        node_kind_match!(
            kind,
            if "➕ Add new".cstr().button(ui).clicked() {
                op(move |world| {
                    NodeType::default().open_publish_window(world, parent);
                });
            }
        );
    }
    pub fn render_node_list(ui: &mut Ui, world: &mut World, kind: NamedNodeKind) -> NodeResult<()> {
        Self::show_add_new(ui, kind.to_kind(), None);

        world.resource_scope::<ExplorerState, _>(|_, mut state| {
            with_incubator_source(|ctx| {
                let mut node_list = named_node_kind_match!(kind, {
                    ctx.world_mut()?
                        .query::<(Entity, &NamedNodeType)>()
                        .iter(ctx.world()?)
                        .map(|(_, n)| (n.id, n.name().to_owned(), n.rating()))
                        .collect_vec()
                });
                node_list.sort_by_key(|(_, _, rating)| *rating);
                // Get inspected item
                let inspected_id = match kind {
                    NamedNodeKind::NUnit => state.inspected_unit,
                    NamedNodeKind::NHouse => state.inspected_house,
                    _ => unreachable!(),
                };

                // Render the list
                node_list
                    .as_list(|(id, name, rating), _ctx, ui| {
                        let color = if inspected_id == Some(*id) {
                            YELLOW
                        } else {
                            colorix().high_contrast_text()
                        };
                        format!(
                            "[b {}] [{} {}]",
                            rating.cstr_expanded(),
                            color.to_hex(),
                            name
                        )
                        .label(ui)
                    })
                    .with_hover(|(id, _name, _rating), _ctx, ui| {
                        if ui.button("Inspect").clicked() {
                            let action = match kind {
                                NamedNodeKind::NUnit => ExplorerAction::InspectUnit(*id),
                                NamedNodeKind::NHouse => ExplorerAction::InspectHouse(*id),
                                _ => unreachable!(),
                            };

                            state.pending_actions.push(action);
                        }
                        render_vote_btns(*id, ui);
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

    fn pane_component<T: ClientNode + FDisplay + FDescription + FTitle>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        parent: u64,
    ) -> NodeResult<()> {
        let kind = T::kind_s();
        if !Self::check_fixed(ui, parent, ContentNodeKind::try_from(kind).unwrap()) {
            return Ok(());
        }
        ui.vertical_centered_justified(|ui| {
            format!("[tw [h2 {}]]", kind.cstr()).label_w(ui);
        });
        ui.separator();
        if parent.fixed_kinds().contains(&kind) {
            let id = ctx.first_child(parent, kind)?;
            return ui
                .vertical_centered_justified(|ui| -> NodeResult<()> {
                    let node = ctx.load::<T>(id)?;
                    if node.kind() == NodeKind::NUnitRepresentation {
                        node.display(ctx, ui);
                        ui.separator();
                    }
                    node.description_cstr(ctx)
                        .cstr_s(CstrStyle::Bold)
                        .label_w(ui);
                    Ok(())
                })
                .inner;
        }
        Self::show_add_new(ui, kind, Some(parent));
        ctx.collect_kind_children(parent, kind)?
            .into_iter()
            .filter_map(|id| ctx.load::<T>(id).ok())
            .sorted_by_key(|n| -n.rating())
            .collect_vec()
            .as_list(|node, ctx, ui| {
                if node.kind() == NodeKind::NUnitRepresentation {
                    node.display(ctx, ui);
                }
                let text = ctx
                    .exec_ref(|ctx| {
                        ctx.set_owner(parent);
                        Ok(node.description_cstr(ctx))
                    })
                    .unwrap();
                format!("[b {}] {text}", node.rating().cstr_expanded()).label_w(ui)
            })
            .with_hover(move |node, _, ui| {
                render_vote_btns(node.id(), ui);
                if cn()
                    .db
                    .creators()
                    .node_id()
                    .find(&node.id())
                    .is_some_and(|n| n.player_id == player_id())
                    && "[red Delete]".cstr().button(ui).clicked()
                {
                    let node_id = node.id();
                    op(move |world| {
                        Confirmation::new("Are you sure you want to delete?")
                            .accept_name("[red Delete]")
                            .cancel_name("Cancel")
                            .content(move |ui, world, button_pressed| {
                                with_incubator_source(|ctx| {
                                    ui.vertical_centered(|ui| -> NodeResult<()> {
                                        ctx.load::<T>(node_id)?.display(ctx, ui);
                                        Ok(())
                                    })
                                    .inner
                                })
                                .ui(ui);
                                if let Some(true) = button_pressed {
                                    cn().reducers
                                        .content_delete_node(node_id)
                                        .notify_error(world);
                                }
                            })
                            .push(world);
                    });
                }
            })
            .compose(ctx, ui);
        Ok(())
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NUnitDescription>(ui, ctx, unit_id)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| Self::pane_component::<NUnitBehavior>(ui, ctx, unit_id))
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| Self::pane_component::<NUnitStats>(ui, ctx, unit_id))
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| {
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
                with_incubator_source(|ctx| {
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
                with_incubator_source(|ctx| {
                    Self::pane_component::<NUnitRepresentation>(ui, ctx, unit_id)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| Self::pane_component::<NHouseColor>(ui, ctx, house_id))
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_magic(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NAbilityMagic>(ui, ctx, house_id)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NAbilityDescription>(ui, ctx, house_id)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_effect(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NAbilityEffect>(ui, ctx, house_id)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_magic(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| Self::pane_component::<NStatusMagic>(ui, ctx, house_id))
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NStatusDescription>(ui, ctx, house_id)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NStatusBehavior>(ui, ctx, house_id)
                })
            } else {
                Ok(())
            }
        })
    }

    fn check_fixed(ui: &mut Ui, parent_id: u64, kind: ContentNodeKind) -> bool {
        let kind = match kind {
            ContentNodeKind::NHouse
            | ContentNodeKind::NUnit
            | ContentNodeKind::NAbilityMagic
            | ContentNodeKind::NHouseColor
            | ContentNodeKind::NUnitDescription
            | ContentNodeKind::NStatusMagic => return true,
            ContentNodeKind::NAbilityDescription => NodeKind::NAbilityMagic,
            ContentNodeKind::NAbilityEffect => NodeKind::NAbilityDescription,
            ContentNodeKind::NStatusDescription => NodeKind::NStatusMagic,
            ContentNodeKind::NStatusBehavior | ContentNodeKind::NStatusRepresentation => {
                NodeKind::NStatusDescription
            }
            ContentNodeKind::NUnitStats
            | ContentNodeKind::NUnitBehavior
            | ContentNodeKind::NUnitRepresentation => NodeKind::NUnitDescription,
        };
        if !parent_id.fixed_kinds().contains(&kind) {
            format!("[b {}] [tw should be fixed first]", kind.cstr()).label_w(ui);
            false
        } else {
            true
        }
    }
}

fn render_vote_btns(id: u64, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if "[red ⬇]".cstr().button(ui).clicked() {
            cn().reducers.content_downvote_node(id).notify_error_op();
        }
        if "[green ⬆]".cstr().button(ui).clicked() {
            cn().reducers.content_upvote_node(id).notify_error_op();
        }
    });
}
