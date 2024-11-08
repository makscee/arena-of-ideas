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
        let (command, id) = extra.split('/').next_tuple().unwrap();
        if command == "pass" {
            let id = u64::from_str(id).unwrap();
            cn().reducers.admin_set_temp_pass(id).unwrap();
            cn().reducers.on_admin_set_temp_pass(|e, id| {
                let id = *id;
                info!("Set temp password for player#{id}");
                e.event.on_success(move |w| {
                    info!("Password successfully set for {id}");
                    app_exit(w);
                });
            });
        }
    }
}
