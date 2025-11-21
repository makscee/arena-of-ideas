use spacetimedb_sdk::Table;

use crate::{login, stdb_auth::SpaceLogin};

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
        app.add_systems(OnEnter(GameState::Login), Self::login)
            .init_resource::<LoginData>();
    }
}

#[derive(Resource, Default)]
pub struct LoginData {
    login_requested: bool,
    pub id_token: Option<String>,
}

impl LoginPlugin {
    fn login() {
        cn().reducers.on_login_by_identity(|e| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                op(|world| {
                    world.resource_mut::<LoginData>().login_requested = false;
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
                    world.resource_mut::<LoginData>().login_requested = false;
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
    pub fn pane_login(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            let mut ld = world.resource_mut::<LoginData>();
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let cs = pd().client_state.clone();
            if let Some((name, identity)) = &cs.last_logged_in {
                format!("Login as {name}")
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading2)
                    .label(ui);
                if (pd().client_settings.auto_login
                    || Button::new("Login")
                        .enabled(!ld.login_requested)
                        .ui(ui)
                        .clicked())
                    && !ld.login_requested
                {
                    ld.login_requested = true;
                    //let _ = cn().reducers.login_by_identity();
                }
                br(ui);
                if Button::new("Logout")
                    .enabled(!ld.login_requested)
                    .gray(ui)
                    .ui(ui)
                    .clicked()
                    || ConnectOption::get(world).identity != *identity
                {
                    pd_mut(|data| data.client_state.last_logged_in = None);
                }
            } else {
                "Arena of Ideas"
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading)
                    .label(ui);
                space(ui);
                br(ui);
                if Button::new("Login via SpacetimeAuth").ui(ui).clicked() {
                    world.trigger(SpaceLogin);
                }
            }
        });
    }
}
