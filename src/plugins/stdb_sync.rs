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
        let houses = houses().values().map(|h| h.to_strings_root()).collect_vec();
        cn().reducers.sync_assets(global_settings, houses).unwrap();
    }
}
