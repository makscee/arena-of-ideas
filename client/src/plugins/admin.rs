use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
        "⏵".cstr().label(ui);
        let id = "exp_test".into();
        let mut e = ui.ctx().data_mut(|w| {
            w.get_persisted_mut_or::<Expression>(
                id,
                Expression::r#if(
                    Box::new(Expression::greater_then(
                        Box::new(Expression::var(VarName::hp)),
                        Box::new(Expression::i32(0)),
                    )),
                    Box::new(Expression::sum(
                        Box::new(Expression::var(VarName::pwr)),
                        Box::new(Expression::i32(10)),
                    )),
                    Box::new(Expression::zero),
                ),
            )
            .clone()
        });

        with_solid_source(|context| {
            let mut changed = false;
            e.as_recursive_mut(|_context, ui, value| {
                let response = call_on_recursive_value_mut!(value, edit, ui);
                changed |= response.changed();
                response
            })
            .with_layout(RecursiveLayout::Tree { indent: 0.0 })
            .compose(context, ui);
            if changed {
                ui.ctx().data_mut(|w| w.insert_persisted(id, e));
            }
            Ok(())
        })
        .ui(ui);

        fn show_node_with_children(id: u64, ui: &mut Ui, world: &mut World) {
            ui.horizontal(|ui| {
                id.label(ui);
                if let Some(node) = cn().db.nodes_world().id().find(&id) {
                    ui.vertical(|ui| {
                        ui.label(node.data);
                        if "[red delete node]".cstr().button(ui).clicked() {
                            Confirmation::new("Delete Node?")
                                .cancel(|_| {})
                                .accept(move |world| {
                                    cn().reducers.admin_delete_node_recursive(id).notify(world);
                                })
                                .push(world);
                        }
                    });
                } else {
                    "[red node not found]".cstr().label(ui);
                }
            });
        }
        if "Inspect Nodes".cstr().button(ui).clicked() {
            let mut all_nodes = cn().db.nodes_world().iter().collect_vec();
            Window::new("Nodes Inspector", move |ui, _| {
                ui.horizontal(|ui| {
                    format!("Total Nodes: {}", all_nodes.len()).label(ui);
                    if "Refresh".cstr().button(ui).clicked() {
                        all_nodes = cn().db.nodes_world().iter().collect_vec();
                    }
                });
                all_nodes
                    .table()
                    .column(
                        "id",
                        |ctx, ui, t, _| {
                            ui.horizontal(|ui| {
                                t.id.as_empty()
                                    .with_menu()
                                    .add_action("Inspect", |id, _| {
                                        op(move |world| {
                                            Window::new(
                                                format!("Node Inspect {id}"),
                                                move |ui, world| {
                                                    show_node_with_children(id, ui, world);
                                                },
                                            )
                                            .push(world);
                                        });
                                        None
                                    })
                                    .add_dangerous_action("Delete", |id, _| {
                                        op(move |world| {
                                            Confirmation::new("Delete Node?")
                                                .cancel(|_| {})
                                                .accept(move |world| {
                                                    cn().reducers
                                                        .admin_delete_node_recursive(id)
                                                        .notify(world);
                                                })
                                                .push(world);
                                        });
                                        None
                                    })
                                    .compose_with_menu(ctx, ui);
                                t.id.label(ui);
                            });
                            Ok(())
                        },
                        |_, t| Ok(t.id.into()),
                    )
                    .column(
                        "owner",
                        |_, ui, t, _| {
                            t.owner.label(ui);
                            Ok(())
                        },
                        |_, t| Ok(t.owner.into()),
                    )
                    .column(
                        "kind",
                        |_, ui, t, _| {
                            t.kind().as_ref().to_string().label(ui);
                            Ok(())
                        },
                        |_, t| Ok(t.kind().to_string().into()),
                    )
                    .column(
                        "data",
                        |_, ui, t, _| {
                            Label::new(&t.data).truncate().ui(ui);
                            Ok(())
                        },
                        |_, t| Ok(t.data.clone().into()),
                    )
                    .column_initial_width(400.0)
                    .ui(&EMPTY_CONTEXT, ui);
            })
            .expand()
            .push(world);
        }

        if "Insert Match".cstr().button(ui).clicked() {
            cn().reducers.match_insert().unwrap();
        }
        if "Notification Test".cstr().button(ui).clicked() {
            "notify test".notify(world);
            "notify error test".notify_error(world);
        }
        if "Check Phase Completion".cstr().button(ui).clicked() {
            cn().reducers
                .content_check_phase_completion()
                .notify_error_op();
        }
        if "Add 10 Votes".cstr().button(ui).clicked() {
            cn().reducers.admin_add_votes(10).notify_error_op();
        }
        if "Download Node Assets".cstr().button(ui).clicked() {
            match download_world_assets_to_path(&get_world_assets_path()) {
                Ok(count) => {
                    format!("Downloaded {} nodes and links", count).notify(world);
                }
                Err(e) => {
                    format!("Failed to download assets: {}", e).notify_error(world);
                }
            }
        }
        if "Upload Node Assets".cstr().button(ui).clicked() {
            match upload_world_assets_from_path(&get_world_assets_path()) {
                Ok(count) => {
                    format!("Uploaded {} nodes and links", count).notify(world);
                }
                Err(e) => {
                    format!("Failed to upload assets: {}", e).notify_error(world);
                }
            }
        }
    }
}
