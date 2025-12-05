use super::*;

pub struct IncubatorPanes;

impl IncubatorPanes {
    pub fn render_node_list(ui: &mut Ui, world: &mut World, kind: NamedNodeKind) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_, mut state| {
            with_incubator_source(|ctx| {
                named_node_kind_match!(
                    kind,
                    Self::render_suggestion_button::<NamedNodeType>(ui, None).ui(ui)
                );

                let mut node_list = named_node_kind_match!(kind, {
                    ctx.world_mut()?
                        .query::<(Entity, &NamedNodeType)>()
                        .iter(ctx.world()?)
                        .map(|(_, n)| (n.id, n.name().to_owned(), schema::Node::rating(n), n.owner))
                        .collect_vec()
                });
                node_list.sort_by_key(|(_, _, rating, _)| -*rating);
                let inspected_id = match kind {
                    NamedNodeKind::NUnit => state.inspected_unit,
                    NamedNodeKind::NHouse => state.inspected_house,
                    _ => unreachable!(),
                };
                node_list
                    .as_list(|(id, name, rating, owner), _ctx, ui| {
                        let color = if inspected_id == Some(*id) {
                            YELLOW
                        } else {
                            colorix().high_contrast_text()
                        };
                        let marker = if *owner == ID_CORE {
                            "⭐️"
                        } else if id.fixed_kinds().contains(&kind.to_kind()) {
                            "📌"
                        } else {
                            ""
                        };
                        format!(
                            "[b {}] {} [{} {}]",
                            rating.cstr_expanded(),
                            marker,
                            color.to_hex(),
                            name
                        )
                        .label(ui)
                    })
                    .with_hover(|(id, _name, _rating, _), _ctx, ui| {
                        if ui.button("Inspect").clicked() {
                            let action = match kind {
                                NamedNodeKind::NUnit => IncubatorAction::InspectUnit(*id),
                                NamedNodeKind::NHouse => IncubatorAction::InspectHouse(*id),
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

    fn pane_component<T: ClientNode + FDescription>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        base: u64,
        parent: Option<u64>,
    ) -> NodeResult<()> {
        let kind = T::kind_s();

        if parent.is_none() {
            format!("[b {}] [tw parent not found]", kind.cstr()).label_w(ui);
            return Ok(());
        }

        let parent = parent.unwrap();
        let fixed = base.fixed_kinds();
        if kind.component_parent().is_none() {
            return Err(NodeError::custom(format!(
                "[red Wrong component kind {kind}]]"
            )));
        }
        {
            let mut kind = kind;
            while let Some(parent_kind) = kind.component_parent() {
                if !fixed.contains(&parent_kind) {
                    format!("[b {}] [tw should be complete first]", parent_kind.cstr()).label_w(ui);
                    return Ok(());
                }
                kind = parent_kind;
            }
        }

        ui.vertical_centered_justified(|ui| {
            format!("[tw [h2 {}]]", kind.cstr()).label_w(ui);
        });
        ui.separator();

        if fixed.contains(&kind) {
            let id = ctx.first_child_recursive(base, kind)?;
            return ui
                .vertical_centered_justified(|ui| -> NodeResult<()> {
                    let node = ctx.load::<T>(id)?;
                    ui.horizontal(|ui| {
                        node.rating().cstr_expanded().label(ui);
                        render_vote_btns(id, ui);
                    });
                    if node.kind() == NodeKind::NUnitRepresentation {
                        node.display(ctx, ui);
                        ui.separator();
                    }
                    node.description_cstr(ctx).label_w(ui);
                    Ok(())
                })
                .inner;
        }

        Self::render_suggestion_button::<T>(ui, Some(parent)).ui(ui);
        ui.separator();
        Self::render_component_list::<T>(ui, ctx, parent)?;

        Ok(())
    }

    fn render_suggestion_button<T: ClientNode>(ui: &mut Ui, parent: Option<u64>) -> NodeResult<()> {
        if !"[green ➕ Suggest]".cstr().button(ui).clicked() {
            return Ok(());
        }
        let kind = T::kind_s().to_content()?;
        let mut node = T::default();
        Confirmation::new(&format!("Create new {}", kind))
            .accept_name("[green ✅ Create]")
            .cancel_name("Cancel")
            .content(move |ui, _world, button_pressed| {
                match kind {
                    ContentNodeKind::NUnitBehavior => {
                        let mut cn = node.force_cast::<NUnitBehavior>().clone();
                        if cn.reactions.is_empty() {
                            cn.reactions.push(default());
                        }
                        let reaction = cn.reactions.first_mut().unwrap();
                        let mut changed = false;
                        if Selector::ui_enum(&mut reaction.trigger, ui).1.changed() {
                            changed = true;
                        }
                        if Input::new("Unit Description")
                            .ui_string(&mut reaction.effect.description, ui)
                            .changed()
                        {
                            changed = true;
                        }
                        if changed {
                            node.inject_data(&cn.get_data()).unwrap();
                        }
                    }
                    ContentNodeKind::NStatusBehavior => {
                        let mut cn = node.force_cast::<NStatusBehavior>().clone();
                        if cn.reactions.is_empty() {
                            cn.reactions.push(default());
                        }
                        let reaction = cn.reactions.first_mut().unwrap();
                        let mut changed = false;
                        if Selector::ui_enum(&mut reaction.trigger, ui).1.changed() {
                            changed = true;
                        }
                        if Input::new("Status Description")
                            .ui_string(&mut reaction.effect.description, ui)
                            .changed()
                        {
                            changed = true;
                        }
                        if changed {
                            node.inject_data(&cn.get_data()).unwrap();
                        }
                    }
                    ContentNodeKind::NAbilityEffect => {
                        let mut cn = node.force_cast::<NAbilityEffect>().clone();
                        if Input::new("Effect Description")
                            .ui_string(&mut cn.effect.description, ui)
                            .changed()
                        {
                            node.inject_data(&cn.get_data()).unwrap();
                        }
                    }
                    _ => {
                        node.edit(ui);
                    }
                }
                node.display(&EMPTY_CONTEXT, ui);
                if let Some(true) = button_pressed {
                    cn().reducers
                        .content_suggest_node(kind.to_string(), node.get_data(), parent)
                        .notify_error_op();
                }
            })
            .push_op();
        Ok(())
    }

    fn render_component_list<T: ClientNode + FDisplay + FDescription>(
        ui: &mut Ui,
        ctx: &ClientContext,
        parent: u64,
    ) -> NodeResult<()> {
        let kind = T::kind_s();
        let suggestions = ctx
            .collect_kind_children(parent, kind)?
            .into_iter()
            .filter_map(|id| ctx.load_ref::<T>(id).ok())
            .sorted_by_key(|n| -n.rating())
            .collect_vec();
        if suggestions.is_empty() {
            "[tw No suggestions yet]"
                .cstr_s(CstrStyle::Heading2)
                .label(ui);
            return Ok(());
        }

        suggestions
            .as_list(|node, ctx, ui| {
                if node.kind() == NodeKind::NUnitRepresentation {
                    node.display(ctx, ui);
                }
                let desc = ctx
                    .exec_ref(|ctx| {
                        ctx.set_owner(node.id());
                        Ok(node.description_cstr(ctx))
                    })
                    .unwrap_or_default();
                format!("[b {}] {}", node.rating().cstr_expanded(), desc).label_w(ui)
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

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NUnitBehavior>(ui, ctx, unit_id, Some(unit_id))
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| {
                    let parent = ctx.first_child(unit_id, NodeKind::NUnitBehavior).ok();
                    Self::pane_component::<NUnitStats>(ui, ctx, unit_id, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
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
        world.resource_scope::<IncubatorState, _>(|_world, state| {
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
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_incubator_source(|ctx| {
                    let parent = ctx.first_child(unit_id, NodeKind::NUnitBehavior).ok();
                    Self::pane_component::<NUnitRepresentation>(ui, ctx, unit_id, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_house_color(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NHouseColor>(ui, ctx, house_id, Some(house_id))
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_magic(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NAbilityMagic>(ui, ctx, house_id, Some(house_id))
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_effect(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    let parent = ctx.first_child(house_id, NodeKind::NAbilityMagic).ok();
                    Self::pane_component::<NAbilityEffect>(ui, ctx, house_id, parent)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_magic(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    Self::pane_component::<NStatusMagic>(ui, ctx, house_id, Some(house_id))
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<IncubatorState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_incubator_source(|ctx| {
                    let parent = ctx.first_child(house_id, NodeKind::NStatusMagic).ok();
                    Self::pane_component::<NStatusBehavior>(ui, ctx, house_id, parent)
                })
            } else {
                Ok(())
            }
        })
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
