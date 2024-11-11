use super::*;

pub struct AdminPlugin;

impl Plugin for AdminPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Admin), Self::on_enter);
    }
}

impl AdminPlugin {
    fn on_enter() {
        let extra = ARGS.get().unwrap().extra.clone().unwrap();
        let mut parts = extra.split('/').collect_vec();
        let command = parts.remove(0);
        if command == "pass" {
            let id = u64::from_str(parts[0]).unwrap();
            cn().reducers.admin_set_temp_pass(id).unwrap();
            cn().reducers.on_admin_set_temp_pass(|e, id| {
                let id = *id;
                info!("Set temp password for player#{id}");
                e.event.on_success(move |w| {
                    info!("Password successfully set for {id}");
                    app_exit(w);
                });
            });
        } else if command == "tag" {
            let owner = u64::from_str(parts[0]).unwrap();
            let tag = parts[1];
            cn().reducers.admin_give_tag(owner, tag.into()).unwrap();
            cn().reducers.on_admin_give_tag(|e, owner, tag| {
                let owner = *owner;
                info!("Add tag {tag} to {owner}");
                e.event.on_success(move |w| {
                    info!("{}", "Success".green());
                    app_exit(w);
                });
            });
        }
    }
}
