use spacetimedb_lib::ser::serde::SerializeWrapper;
use spacetimedb_sdk::{identity::once_on_connect, once_on_subscription_applied, subscribe};

use super::*;

pub struct QueryPlugin;

impl Plugin for QueryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Query), Self::on_enter);
    }
}

impl QueryPlugin {
    fn on_enter() {
        ConnectPlugin::connect().unwrap();
        once_on_connect(|_, _| {
            subscribe(&["select * from TGlobalEvent", "select * from TUser"]).unwrap();
        });
        once_on_subscription_applied(|| {
            let json = serde_json::to_string_pretty(&SerializeWrapper::new(
                TGlobalEvent::iter().collect_vec(),
            ))
            .unwrap();
            save_to_download_folder("global_events", json);
            let json =
                serde_json::to_string_pretty(&SerializeWrapper::new(TPlayer::iter().collect_vec()))
                    .unwrap();
            save_to_download_folder("users", json);
            app_exit_op();
        });
    }
}
