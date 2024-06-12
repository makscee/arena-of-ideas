use super::*;

pub struct ServerSyncPlugin;

impl Plugin for ServerSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ServerSync), Self::sync);
    }
}

impl ServerSyncPlugin {
    fn sync(world: &mut World) {
        let ga = GameAssets::get(world).clone();
        let gs = ga.global_settings;
        let units = ga.heroes.into_values().map(|u| u.into()).collect_vec();
        let houses = ga.houses.into_values().map(|h| h.into()).collect_vec();
        let abilities = ga.abilities.into_values().map(|a| a.into()).collect_vec();
        let statuses = ga.statuses.into_values().map(|s| s.into()).collect_vec();
        sync_all_assets(gs, units, houses, abilities, statuses);
        app_exit(world);
    }
}
