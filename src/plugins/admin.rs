use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, _: &mut App) {}
}

impl AdminPlugin {
    pub fn pane(ui: &mut Ui, world: &mut World) {
        let id = ui.id();
        if let Some(e) = world.get_id_link(ID_CORE) {
            let context = &Context::new_world(world);
            let houses = context
                .children_components::<House>(e)
                .into_iter()
                .filter_map(|h| House::pack(e, context))
                .collect_vec();
            for h in houses {
                h.view(ViewContext::new(ui), &default(), ui);
            }
        }
        let mut e = ui.data(|r| r.get_temp::<Material>(id)).unwrap_or_default();
        if e.view_mut(ViewContext::new(ui), &default(), ui).changed {
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
        if "Export Players".cstr().button(ui).clicked() {
            let context = &Context::new_world(world);
            let players = Players::pack(world.get_id_link(ID_PLAYERS).unwrap(), context).unwrap();
            dbg!(&players);
            let dir = players.to_dir("players".into());
            let dir = Dir::new("players", dir);
            let path = "./assets/ron/";
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Export Core".cstr().button(ui).clicked() {
            let context = &Context::new_world(world);
            let core = Core::pack(world.get_id_link(ID_CORE).unwrap(), context).unwrap();
            dbg!(&core);
            let dir = core.to_dir("core".into());
            let dir = Dir::new("core", dir);
            let path = "./assets/ron/";
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Export Incubator".cstr().button(ui).clicked() {
            let context = &Context::new_world(world);
            let inc = Incubator::pack(world.get_id_link(ID_INCUBATOR).unwrap(), context).unwrap();
            dbg!(&inc);
            let dir = inc.to_dir("incubator".into());
            let dir = Dir::new("incubator", dir);
            let path = "./assets/ron/";
            std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
            dir.extract(path).unwrap();
        }
        if "Export Incubator Data".cstr().button(ui).clicked() {
            GameAssets::update_files();
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
