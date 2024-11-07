use spacetimedb_lib::ser::serde::SerializeWrapper;
use spacetimedb_sdk::DbContext;

use super::*;

pub struct QueryPlugin;

impl Plugin for QueryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Query), Self::on_enter);
    }
}

impl QueryPlugin {
    fn on_enter() {
        ConnectPlugin::connect(|cn, _, _| {
            cn.subscription_builder()
                .on_applied(|e| {
                    let json = serde_json::to_string_pretty(&SerializeWrapper::new(
                        e.db.global_event().iter().collect_vec(),
                    ))
                    .unwrap();
                    save_to_download_folder("global_events", json);
                    let json = serde_json::to_string_pretty(&SerializeWrapper::new(
                        e.db.player().iter().collect_vec(),
                    ))
                    .unwrap();
                    save_to_download_folder("users", json);
                    app_exit_op();
                })
                .subscribe(["select * from TGlobalEvent", "select * from TUser"]);
        });
    }
}
