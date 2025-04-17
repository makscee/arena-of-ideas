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
        let core = NCore::from_dir(0, "core".into(), assets()).unwrap();
        let core = core.to_tnodes();
        let incubator = NIncubator::from_dir(0, "incubator".into(), assets()).unwrap();
        let incubator = incubator.to_tnodes();
        let players = NPlayers::from_dir(0, "players".into(), assets()).unwrap();
        let players = players.to_tnodes();
        let inc = IncubatorData::load();
        cn().reducers.on_sync_assets(|e, _, _, _, _, _, _, _| {
            if !e.check_identity() {
                return;
            }
            e.event.notify_error();
            info!("{}", "Assets sync done".blue());
            app_exit_op();
        });
        let incubator_nodes = inc.nodes.into_iter().map(|n| n.0).collect_vec();
        let incubator_links = inc.links.into_iter().map(|n| n.0).collect_vec();
        let incubator_votes = inc.votes.into_iter().map(|n| n.0).collect_vec();
        cn().reducers
            .sync_assets(
                global_settings,
                core,
                players,
                incubator,
                incubator_nodes,
                incubator_links,
                incubator_votes,
            )
            .unwrap();
    }
}
