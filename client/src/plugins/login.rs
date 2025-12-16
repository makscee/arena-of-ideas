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

#[derive(Resource, Default)]
pub struct LoginData {
    pub user_exists: bool,
    pub username: String,
}

impl LoginPlugin {
    fn login() {
        cn().reducers.on_login_by_identity(|e| {
            if !e.check_identity() {
                return;
            }
            match &e.event.status {
                spacetimedb_sdk::Status::Committed => {
                    op(|world| {
                        if let Some(mut ld) = world.get_resource_mut::<LoginData>() {
                            ld.user_exists = true;
                        }
                    });
                    e.event.on_success_error(LoginPlugin::complete, || {
                        info!("Logging in with identity");
                        pd_mut(|pd| pd.client_state.last_logged_in = None);
                    });
                }
                spacetimedb_sdk::Status::Failed(err) => {
                    let err_str = err.to_string();
                    if err_str.contains("NO_IDENTITY_FOUND") {
                        info!("No player found for identity");
                    }
                }
                _ => panic!(),
            }
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
    pub fn pane_login(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let mut ld = world.resource_mut::<LoginData>();
            if !ld.user_exists {
                "Register New Player"
                    .cstr_cs(high_contrast_text(), CstrStyle::Heading2)
                    .label(ui);

                Input::new("username").ui_string(&mut ld.username, ui);
                if Button::new("Register").ui(ui).clicked() {
                    cn().reducers.on_register(|e, _| {
                        if !e.check_identity() {
                            return;
                        }
                        e.event.on_success(LoginPlugin::complete);
                    });
                    cn().reducers.register(ld.username.clone()).unwrap();
                }
            }
        });
    }
}
