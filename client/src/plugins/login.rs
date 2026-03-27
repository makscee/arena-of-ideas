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
        cn().reducers.login_by_identity_then(|_ctx, result| {
            match result {
                Ok(Ok(())) => {
                    op(|world| {
                        if let Some(mut ld) = world.get_resource_mut::<LoginData>() {
                            ld.user_exists = true;
                        }
                    });
                    LoginPlugin::complete();
                }
                Ok(Err(err)) => {
                    if err.contains("not found") {
                        info!("No player found for identity");
                    } else {
                        err.notify_error_op();
                    }
                    pd_mut(|pd| pd.client_state.last_logged_in = None);
                }
                Err(e) => {
                    format!("{e:?}").notify_error_op();
                }
            }
        }).ok();
    }
    pub fn complete() {
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
                    let username = ld.username.clone();
                    cn().reducers.register_then(username, |_ctx, result| {
                        match result {
                            Ok(Ok(())) => LoginPlugin::complete(),
                            Ok(Err(e)) => e.notify_error_op(),
                            Err(e) => format!("{e:?}").notify_error_op(),
                        }
                    }).ok();
                }
            }
        });
    }
}
