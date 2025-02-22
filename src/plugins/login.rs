use spacetimedb_sdk::Table;

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
        app.add_systems(OnEnter(GameState::Login), Self::login)
            .init_resource::<LoginData>();
    }
}

#[derive(Resource, Default)]
pub struct LoginData {
    name_field: String,
    pass_field: String,
    pub identity_player: Option<TPlayer>,
    login_requested: bool,
}

impl LoginPlugin {
    fn login(world: &mut World) {
        let co = ConnectOption::get(world);
        let mut identity_user = None;
        if let Some(player) = cn()
            .db
            .player()
            .iter()
            .find(|u| u.identities.contains(&co.identity))
        {
            if currently_fulfilling() == GameOption::ForceLogin {
                Self::complete(Some(player.clone()), world);
            }
            identity_user = Some(player);
        }
        world.insert_resource(LoginData {
            identity_player: identity_user,
            ..default()
        });
    }
    pub fn complete(player: Option<TPlayer>, world: &mut World) {
        let player = player.unwrap_or_else(|| {
            world
                .resource::<LoginData>()
                .identity_player
                .clone()
                .unwrap()
        });
        LoginOption { player }.save(world);
        subscribe_reducers();
        subscribe_game(GameState::proceed_op);
    }
    pub fn tab(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let mut ld = world.resource_mut::<LoginData>();
            if let Some(player) = ld.identity_player.clone() {
                format!("Login as {}", player.name)
                    .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                    .label(ui);
                if !ld.login_requested
                    && (client_settings().auto_login || Button::new("Login").ui(ui).clicked())
                {
                    ld.login_requested = true;
                    cn().reducers.on_login_by_identity(|e| {
                        if !e.check_identity() {
                            return;
                        }
                        e.event.on_success_op(|world| {
                            LoginPlugin::complete(None, world);
                        });
                    });
                    let _ = cn().reducers.login_by_identity();
                }
                br(ui);
                if Button::new("Logout").gray(ui).ui(ui).clicked() {
                    ld.identity_player = None;
                }
            } else {
                let mut ld = world.resource_mut::<LoginData>();
                "Register"
                    .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading)
                    .label(ui);
                if Button::new("New Player").ui(ui).clicked() {
                    cn().reducers.on_register_empty(|e| {
                        if !e.check_identity() {
                            return;
                        }
                        e.event.on_success_op(|world| {
                            Notification::new_string("New player created".to_owned()).push(world);
                            let identity = ConnectOption::get(world).identity.clone();
                            let player = cn()
                                .db
                                .player()
                                .iter()
                                .find(|u| u.identities.contains(&identity))
                                .expect("Failed to find player after registration");
                            world.resource_mut::<LoginData>().identity_player = Some(player);
                        })
                    });
                    cn().reducers.register_empty().unwrap();
                }
                br(ui);
                "Login".cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading).label(ui);
                Input::new("name").ui_string(&mut ld.name_field, ui);
                Input::new("password")
                    .password()
                    .ui_string(&mut ld.pass_field, ui);
                if Button::new("Submit").ui(ui).clicked() {
                    cn().reducers.on_login(|e, name, _| {
                        if !e.check_identity() {
                            return;
                        }
                        let name = name.clone();
                        let player = e.db.player().name().find(&name).unwrap();
                        e.event.on_success_op(move |world| {
                            LoginPlugin::complete(Some(player), world);
                        })
                    });
                    let _ = crate::login::login(
                        &cn().reducers,
                        ld.name_field.clone(),
                        ld.pass_field.clone(),
                    );
                }
            }
        });
    }
}
