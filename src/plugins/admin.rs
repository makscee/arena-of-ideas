use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
        let id = ui.id();
        if let Some(e) = world.get_id_link(ID_CORE) {
            let context = &Context::new(world);
            let houses = context
                .children_components::<NHouse>(e)
                .into_iter()
                .filter_map(|h| NHouse::pack_entity(e, context))
                .collect_vec();
            for h in houses {
                h.view(ViewContext::new(ui), &default(), ui);
            }
        }
        let mut e = ui.data(|r| r.get_temp::<Material>(id)).unwrap_or_default();
        if e.view_mut(ViewContext::new(ui), &default(), ui).changed {
            ui.data_mut(|w| w.insert_temp(id, e));
        }

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
                .filter(|n| n.id.is_child_of(id))
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
                let context = &Context::new(world);
                let view_ctx = ViewContext::new(ui).collapsed(true);
                if let Some(core) = NCore::get_by_id(ID_CORE, context) {
                    core.view(view_ctx, context, ui);
                }
                if let Some(players) = NPlayers::get_by_id(ID_PLAYERS, context) {
                    players.view(view_ctx, context, ui);
                }
            })
            .push(world);
        }
        if "Export Players".cstr().button(ui).clicked() {
            let context = &Context::new(world);
            let players =
                NPlayers::pack_entity(world.get_id_link(ID_PLAYERS).unwrap(), context).unwrap();
            dbg!(&players);
            let dir = players.to_dir("players".into());
            let dir = Dir::new("players", dir);
            let path = "./assets/ron/";
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Export NCore".cstr().button(ui).clicked() {
            let context = &Context::new(world);
            let core = NCore::pack_entity(world.get_id_link(ID_CORE).unwrap(), context).unwrap();
            dbg!(&core);
            let dir = core.to_dir("core".into());
            let dir = Dir::new("core", dir);
            let path = "./assets/ron/";
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Notification Test".cstr().button(ui).clicked() {
            "notify test".notify(world);
            "notify error test".notify_error(world);
        }
    }
}
