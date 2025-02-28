use super::*;

pub struct StdbSyncPlugin;

impl Plugin for StdbSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ServerSync), Self::sync_assets);
    }
}

impl StdbSyncPlugin {
    fn sync_assets() {
        info!("{}", "Start assets sync".blue());
        let global_settings = global_settings_local().clone();
        let all = all().to_strings_root();
        cn().reducers.on_sync_assets(|e, _, _| {
            if !e.check_identity() {
                return;
            }
            e.event.notify_error();
            info!("{}", "Assets sync done".blue());
            app_exit_op();
        });
        cn().reducers.sync_assets(global_settings, all).unwrap();
    }
}
