use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
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

        let action_id = "action_test".into();
        let mut a = ui.ctx().data_mut(|w| {
            w.get_persisted_mut_or::<Action>(action_id, Action::default())
                .clone()
        });

        fn handle_field_change(
            field: &mut RecursiveFieldMut,
            context: &Context,
            ui: &mut Ui,
        ) -> bool {
            match &mut field.value {
                RecursiveValueMut::Expr(e) => {
                    if let Some(n) = (**e)
                        .see_mut(context)
                        .ctxbtn()
                        .add_paste()
                        .add_copy()
                        .ui_enum(ui)
                        .selector_changed()
                    {
                        RecursiveValueMut::replace_expr_and_move_fields(e, n.clone());
                        true
                    } else {
                        false
                    }
                }
                RecursiveValueMut::Action(action) => {
                    if let Some(n) = (**action)
                        .see_mut(context)
                        .ctxbtn()
                        .ui_enum(ui)
                        .selector_changed()
                    {
                        RecursiveValueMut::replace_action_and_move_fields(action, n.clone());
                        true
                    } else {
                        false
                    }
                }
                RecursiveValueMut::PainterAction(painter_action) => {
                    if let Some(n) = (**painter_action)
                        .see_mut(context)
                        .ctxbtn()
                        .ui_enum(ui)
                        .selector_changed()
                    {
                        RecursiveValueMut::replace_painter_action_and_move_fields(
                            painter_action,
                            n.clone(),
                        );
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }

        Context::from_world(world, |context| {
            let mut changed = false;
            e.render_mut(context)
                .recursive_mut(ui, |ui, context, field| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            format!("[tw [s {}]]", field.name).label(ui);
                            changed = true;
                            handle_field_change(field, context, ui)
                        })
                    })
                    .inner
                    .inner
                });
            if changed {
                ui.ctx().data_mut(|w| w.insert_persisted(id, e))
            }
        });

        fn show_node_with_children(id: u64, ui: &mut Ui, world: &mut World) {
            ui.horizontal(|ui| {
                format!("#[tw {id}]").cstr().label(ui);
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
                }
            });

            for n in cn()
                .db
                .nodes_world()
                .iter()
                .filter(|n| n.id.is_child_of(world, id))
                .sorted_by_key(|n| n.id)
            {
                let title = n.kind;
                CollapsingHeader::new(title).id_salt(n.id).show(ui, |ui| {
                    show_node_with_children(n.id, ui, world);
                });
            }
        }
        if "Inspect Nodes".cstr().button(ui).clicked() {
            Window::new("Nodes Inspector", |ui, world| {
                show_node_with_children(1, ui, world);
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
        if "Rotate Content".cstr().button(ui).clicked() {
            cn().reducers.on_content_rotation(|c| {
                c.event.notify_error();
            });
            cn().reducers.content_rotation().unwrap();
        }
        if "Download Node Assets".cstr().button(ui).clicked() {
            let mut manager = load_node_assets().unwrap();
            for node in cn().db.nodes_world().iter() {
                manager.add_node_from_tnode(&node);
            }
            for link in cn().db.node_links().iter() {
                manager.add_link_from_tnode_link(&link);
            }
            manager.save_to_files().unwrap();
        }
        if "Upload Node Assets".cstr().button(ui).clicked() {
            let manager = load_node_assets().unwrap();
            let mut all_nodes: Vec<(u64, String, NodeAsset)> = default();
            let mut all_links: Vec<LinkAsset> = default();
            for (kind, nodes) in manager.nodes {
                let kind = kind.to_string();
                for (id, node) in nodes {
                    all_nodes.push((id, kind.clone(), node));
                }
            }
            for link in manager.links {
                all_links.push(link);
            }
            let all_nodes = all_nodes
                .into_iter()
                .map(|n| ron::to_string(&n).unwrap())
                .collect_vec();
            let all_links = all_links
                .into_iter()
                .map(|l| ron::to_string(&l).unwrap())
                .collect_vec();
            cn().reducers
                .admin_upload_world(global_settings().clone(), all_nodes, all_links)
                .unwrap();
        }
    }
}
