use crate::login;

use super::*;

pub const HOME_DIR: &str = ".aoi";
pub fn home_dir_path() -> PathBuf {
    let mut path = home::home_dir().unwrap();
    path.push(HOME_DIR);
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Login), Self::login);
    }
}

impl LoginPlugin {
    fn login() {
        cn().reducers.on_login_by_identity(|e| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                op(|world| {
                    world.resource_mut::<AuthOption>().id_token = None;
                    pd_mut(|pd| pd.client_state.last_logged_in = None);
                });
            });
        });
        cn().reducers.on_login(|e, _, _| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                op(|world| {
                    world.resource_mut::<AuthOption>().id_token = None;
                    pd_mut(|pd| pd.client_state.last_logged_in = None);
                });
            });
        });
    }
    fn complete() {
        subscribe_game(Self::on_subscribed);
        subscribe_reducers();
    }
    fn find_identity_node(identity: Identity) -> Option<NPlayerIdentity> {
        info!("Finding identity node for identity: {}", identity);
        let data = NPlayerIdentity::new(0, 0, Some(identity.to_string())).get_data();
        cn().db
            .nodes_world()
            .iter()
            .find(|n| n.data == data)
            .map(|n| n.to_node().unwrap())
    }
    fn on_subscribed() {
        op(|world| {
            let identity = ConnectOption::get(world).identity;
            let Some(identity_node) = Self::find_identity_node(identity) else {
                "Failed to find Player after login".notify_error(world);
                return;
            };
            dbg!(&identity_node);

            if let Some(player) = cn()
                .db
                .node_links()
                .iter()
                .find(|link| link.child == identity_node.id)
                .and_then(|link| NPlayer::db_load(link.parent).ok())
            {
                pd_mut(|pd| {
                    pd.client_state.last_logged_in = Some((player.player_name.clone(), identity));
                });
                LoginOption { player }.save(world);
            } else {
                error!("Failed to load NPlayer by Identity");
            }
            GameState::proceed(world);
        });
    }
}
