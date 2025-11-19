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
            with_solid_source(|ctx| {
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

    fn pane_component<T: ClientNode + FDisplay + FDescription, P: ClientNode>(
        ui: &mut Ui,
        ctx: &mut ClientContext,
        owner: u64,
        parent: P,
        filter: Option<fn(&T, &P) -> bool>,
    ) -> NodeResult<()> {
        let kind = T::kind_s();
        Self::show_add_new(ui, kind);
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
            .sorted_by_key(|n| -n.rating())
            .collect_vec()
            .as_list(|node, ctx, ui| {
                let text = ctx
                    .exec_ref(|ctx| {
                        ctx.set_owner(owner);
                        Ok(node.description_cstr(ctx))
                    })
                    .unwrap();
                format!("[b {}] {text}", node.rating().cstr_expanded()).label_w(ui)
            })
            .with_hover(move |node, _, ui| {
                ui.horizontal(|ui| {
                    if "[red ⬇]".cstr().button(ui).clicked() {
                        cn().reducers
                            .content_downvote_node(node.id())
                            .notify_error_op();
                    }
                    if "[green ⬆]".cstr().button(ui).clicked() {
                        cn().reducers
                            .content_upvote_node(node.id())
                            .notify_error_op();
                    }
                });
            })
            .compose(ctx, ui);
        Ok(())
    }

    pub fn pane_unit_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_solid_source(|ctx| {
                    // Check if NUnitDescription is fixed in creation phases
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&unit_id) {
                        if phases.fixed_kinds.contains(&"NUnitDescription".to_string()) {
                            // If fixed, display the single node
                            if let Some(desc) =
                                ctx.load_children_ref::<NUnitDescription>(unit_id)?.first()
                            {
                                desc.display(ctx, ui);
                            }
                            return Ok(());
                        }
                    }
                    // If not fixed, show list and add button
                    let parent = ctx.load::<NUnit>(unit_id)?;
                    Self::pane_component::<NUnitDescription, _>(ui, ctx, unit_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_solid_source(|ctx| {
                    // Check if NUnitDescription is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&unit_id) {
                        if !phases.fixed_kinds.contains(&"NUnitDescription".to_string()) {
                            ui.label("NUnitDescription must be fixed first");
                            return Ok(());
                        }

                        // Check if NUnitBehavior is fixed
                        if phases.fixed_kinds.contains(&"NUnitBehavior".to_string()) {
                            ctx.load_children_ref::<NUnitBehavior>(unit_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                    } else {
                        ui.label("NUnitDescription must be fixed first");
                        return Ok(());
                    }

                    // If description is fixed but behavior is not, show list for voting
                    let parent = ctx.load::<NUnit>(unit_id)?.description_ref(ctx)?.clone();
                    Self::pane_component::<NUnitBehavior, _>(ui, ctx, unit_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_stats(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_solid_source(|ctx| {
                    // Check if NUnitDescription is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&unit_id) {
                        if !phases.fixed_kinds.contains(&"NUnitDescription".to_string()) {
                            ui.label("NUnitDescription must be fixed first");
                            return Ok(());
                        }

                        // Check if NUnitStats is fixed
                        if phases.fixed_kinds.contains(&"NUnitStats".to_string()) {
                            ctx.load_children_ref::<NUnitStats>(unit_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                    } else {
                        ui.label("NUnitDescription must be fixed first");
                        return Ok(());
                    }

                    // If description is fixed but stats is not, show list for voting
                    let parent = ctx.load::<NUnit>(unit_id)?;
                    Self::pane_component::<NUnitStats, _>(ui, ctx, unit_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_unit_card(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(unit_id) = state.inspected_unit {
                with_solid_source(|ctx| {
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
                with_solid_source(|ctx| {
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
                with_solid_source(|ctx| {
                    // Check if NUnitDescription is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&unit_id) {
                        if !phases.fixed_kinds.contains(&"NUnitDescription".to_string()) {
                            ui.label("NUnitDescription must be fixed first");
                            return Ok(());
                        }

                        // Check if NUnitRepresentation is fixed
                        if phases
                            .fixed_kinds
                            .contains(&"NUnitRepresentation".to_string())
                        {
                            ctx.load_children_ref::<NUnitRepresentation>(unit_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                    } else {
                        ui.label("NUnitDescription must be fixed first");
                        return Ok(());
                    }

                    // If description is fixed but representation is not, show list for voting
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
                with_solid_source(|ctx| {
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

    pub fn pane_ability_magic(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_solid_source(|ctx| {
                    // AbilityMagic has no prerequisites
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&house_id) {
                        if phases.fixed_kinds.contains(&"NAbilityMagic".to_string()) {
                            ctx.load_children_ref::<NAbilityMagic>(house_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                    }

                    // Show list for voting and adding new
                    let parent = ctx.load::<NHouse>(house_id)?;
                    Self::pane_component::<NAbilityMagic, _>(ui, ctx, house_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_solid_source(|ctx| {
                    // Check if NAbilityMagic is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&house_id) {
                        if !phases.fixed_kinds.contains(&"NAbilityMagic".to_string()) {
                            ui.label("NAbilityMagic must be fixed first");
                            return Ok(());
                        }

                        // Check if NAbilityDescription is fixed
                        if phases
                            .fixed_kinds
                            .contains(&"NAbilityDescription".to_string())
                        {
                            ctx.load_children_ref::<NAbilityDescription>(house_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        } else {
                            let parent = ctx.load::<NHouse>(house_id)?;
                            return Self::pane_component::<NAbilityDescription, _>(
                                ui, ctx, house_id, parent, None,
                            );
                        }
                    } else {
                        ui.label("NAbilityMagic must be fixed first");
                        return Ok(());
                    }
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_ability_effect(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_solid_source(|ctx| {
                    // Check if NAbilityDescription is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&house_id) {
                        if !phases
                            .fixed_kinds
                            .contains(&"NAbilityDescription".to_string())
                        {
                            ui.label("NAbilityDescription must be fixed first");
                            return Ok(());
                        }

                        // Check if NAbilityEffect is fixed
                        if phases.fixed_kinds.contains(&"NAbilityEffect".to_string()) {
                            ctx.load_children_ref::<NAbilityEffect>(house_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                        let parent = ctx.load::<NHouse>(house_id)?;
                        Self::pane_component::<NAbilityEffect, _>(ui, ctx, house_id, parent, None)
                    } else {
                        ui.label("NAbilityDescription must be fixed first");
                        return Ok(());
                    }
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_magic(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_solid_source(|ctx| {
                    // StatusMagic has no prerequisites
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&house_id) {
                        if phases.fixed_kinds.contains(&"NStatusMagic".to_string()) {
                            ctx.load_children_ref::<NStatusMagic>(house_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                    }

                    // Show list for voting and adding new
                    let parent = ctx.load::<NHouse>(house_id)?;
                    Self::pane_component::<NStatusMagic, _>(ui, ctx, house_id, parent, None)
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_description(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_solid_source(|ctx| {
                    // Check if NStatusMagic is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&house_id) {
                        if !phases.fixed_kinds.contains(&"NStatusMagic".to_string()) {
                            ui.label("NStatusMagic must be fixed first");
                            return Ok(());
                        }

                        // Check if NStatusDescription is fixed
                        if phases
                            .fixed_kinds
                            .contains(&"NStatusDescription".to_string())
                        {
                            ctx.load_children_ref::<NStatusDescription>(house_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }

                        let parent = ctx.load::<NHouse>(house_id)?;
                        Self::pane_component::<NStatusDescription, _>(
                            ui, ctx, house_id, parent, None,
                        )
                    } else {
                        ui.label("NStatusMagic must be fixed first");
                        return Ok(());
                    }
                })
            } else {
                Ok(())
            }
        })
    }

    pub fn pane_status_behavior(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope::<ExplorerState, _>(|_world, state| {
            if let Some(house_id) = state.inspected_house {
                with_solid_source(|ctx| {
                    // Check if NStatusDescription is fixed first
                    if let Some(phases) = cn().db.creation_phases().node_id().find(&house_id) {
                        if !phases
                            .fixed_kinds
                            .contains(&"NStatusDescription".to_string())
                        {
                            ui.label("NStatusDescription must be fixed first");
                            return Ok(());
                        }

                        // Check if NStatusBehavior is fixed
                        if phases.fixed_kinds.contains(&"NStatusBehavior".to_string()) {
                            ctx.load_children_ref::<NStatusBehavior>(house_id)?
                                .first()
                                .unwrap()
                                .display(ctx, ui);
                            return Ok(());
                        }
                        let parent = ctx.load::<NHouse>(house_id)?;
                        Self::pane_component::<NStatusBehavior, _>(ui, ctx, house_id, parent, None)
                    } else {
                        ui.label("NStatusDescription must be fixed first");
                        return Ok(());
                    }
                })
            } else {
                Ok(())
            }
        })
    }
}
