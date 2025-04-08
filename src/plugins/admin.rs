use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
        let id = ui.id();
        if let Some(e) = world.get_id_link(ID_CORE) {
            let houses = Context::new_world(world)
                .children_components::<House>(e)
                .into_iter()
                .filter_map(|h| House::pack(e, world))
                .collect_vec();
            for h in houses {
                h.view(DataViewContext::new(ui), &default(), ui);
            }
        }
        let mut e = ui.data(|r| r.get_temp::<Material>(id)).unwrap_or_default();
        if e.view_mut(DataViewContext::new(ui), &default(), ui) {
            ui.data_mut(|w| w.insert_temp(id, e));
        }

        if "Insert Match".cstr().button(ui).clicked() {
            cn().reducers.match_insert().unwrap();
        }
        if "Houses Editor".cstr().button(ui).clicked() {
            GameAssetsEditor::open_houses_window(world);
        }
        if "Incubator Merge".cstr().button(ui).clicked() {
            cn().reducers.incubator_merge().unwrap();
        }
        if "Export Core".cstr().button(ui).clicked() {
            let all = Core::pack(world.get_id_link(ID_CORE).unwrap(), world).unwrap();
            dbg!(&all);
            let path = "./assets/";
            let dir = all.to_dir("ron".into());
            let dir = dir.as_dir().unwrap();
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Export Incubator Data".cstr().button(ui).clicked() {
            GameAssets::update_files();
        }
        let r = "Context Test".cstr().button(ui);
        ContextMenu::new(r)
            .add("test1", |ui, _| {
                debug!("test1");
            })
            .add("test2", |ui, _| {
                debug!("test2");
            })
            .add("test3", |ui, _| {
                debug!("test3");
            })
            .ui(ui, world);
        if "Add Team Editor Panes".cstr().button(ui).clicked() {
            TeamEditorPlugin::load_team(default(), world);
            TeamEditorPlugin::add_panes();
            TeamEditorPlugin::unit_add_from_core(world).notify(world);
        }
        if "Notification Test".cstr().button(ui).clicked() {
            "notify test".notify(world);
            "notify error test".notify_error(world);
        }
        if "Incubator".cstr().button(ui).clicked() {
            GameState::Incubator.set_next(world);
        }
    }
}
