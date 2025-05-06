use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
        let id = "exp_test".into();
        let mut e = ui
            .ctx()
            .data_mut(|w| w.get_persisted_mut_or_default::<Expression>(id).clone());

        Context::from_world(world, |context| {
            if e.view_with_children_mut(ViewContext::new(ui), context, ui)
                .changed
            {
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
                show_node_with_children(0, ui, world);
            })
            .expand()
            .push(world);
        }

        if "Insert Match".cstr().button(ui).clicked() {
            cn().reducers.match_insert().unwrap();
        }
        if "World Inspector".cstr().button(ui).clicked() {
            Window::new("world Inspector", |ui, world| {
                Context::from_world_r(world, |context| {
                    let vctx = ViewContext::new(ui);
                    NCore::get_by_id(ID_CORE, context)?.view(vctx, context, ui);
                    NPlayers::get_by_id(ID_PLAYERS, context)?.view(vctx, context, ui);
                    Ok(())
                })
                .unwrap();
            })
            .push(world);
        }
        if "Export Players".cstr().button(ui).clicked() {
            Context::from_world_r(world, |context| {
                let players = NPlayers::pack_entity(context, context.entity(ID_PLAYERS)?)?;
                dbg!(&players);
                let dir = players.to_dir("players".into());
                let dir = Dir::new("players", dir);
                let path = "./assets/ron/";
                std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
                dir.extract(path).unwrap();
                Ok(())
            })
            .unwrap();
        }
        if "Export NCore".cstr().button(ui).clicked() {
            Context::from_world_r(world, |context| {
                let core = NCore::pack_entity(context, context.entity(ID_CORE)?)?;
                dbg!(&core);
                let dir = core.to_dir("core".into());
                let dir = Dir::new("core", dir);
                let path = "./assets/ron/";
                std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
                dir.extract(path).unwrap();
                Ok(())
            })
            .unwrap();
        }
        if "Notification Test".cstr().button(ui).clicked() {
            "notify test".notify(world);
            "notify error test".notify_error(world);
        }
    }
}
