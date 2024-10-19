use spacetimedb_sdk::{
    once_on_subscription_applied,
    table::{TableType, TableWithPrimaryKey},
};

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
struct LoginData {
    name_field: String,
    pass_field: String,
    identity_user: Option<TUser>,
}

impl LoginPlugin {
    fn login(world: &mut World) {
        let co = ConnectOption::get(world);
        let mut identity_user = None;
        if let Some(user) = TUser::iter().find(|u| u.identities.contains(&co.creds.identity)) {
            if matches!(currently_fulfilling(), GameOption::ForceLogin) {
                Self::complete(user.clone(), world);
            }
            identity_user = Some(user);
        }
        world.insert_resource(LoginData {
            name_field: default(),
            pass_field: default(),
            identity_user,
        });
    }
    fn complete(user: TUser, world: &mut World) {
        StdbQuery::Game(user.id).subscribe();
        LoginOption { user }.save(world);
        once_on_subscription_applied(|| {
            OperationsPlugin::add(|world| {
                GameAssets::cache_tables();
                GameState::proceed(world);
            });
            TWallet::on_update(|before, after, _| {
                let delta = after.amount - before.amount;
                let delta_txt = if delta > 0 {
                    format!("+{delta}")
                } else {
                    delta.to_string()
                };
                AudioPlugin::queue_sound(SoundEffect::Coin);
                Notification::new(
                    "Credits "
                        .cstr_c(YELLOW)
                        .push(delta_txt.cstr_c(visible_light()))
                        .take(),
                )
                .push_op();
            });
        });
    }
    pub fn login_ui(ui: &mut Ui, world: &mut World) {
        center_window("login", ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                let mut ld = world.resource_mut::<LoginData>();
                if let Some(user) = ld.identity_user.clone() {
                    format!("Login as {}", user.name)
                        .cstr_cs(visible_light(), CstrStyle::Heading2)
                        .label(ui);
                    if Button::click("Login").ui(ui).clicked() {
                        login_by_identity();
                        once_on_login_by_identity(|_, _, status| {
                            match status {
                                spacetimedb_sdk::reducer::Status::Committed => {
                                    OperationsPlugin::add(|world| {
                                        Self::complete(user, world);
                                    });
                                }
                                spacetimedb_sdk::reducer::Status::Failed(e) => {
                                    Notification::new_string(format!("Login failed: {e}")).push_op()
                                }
                                _ => panic!(),
                            };
                        });
                    }
                    br(ui);
                    if Button::click("Logout").gray(ui).ui(ui).clicked() {
                        ld.identity_user = None;
                    }
                } else {
                    let mut ld = world.resource_mut::<LoginData>();
                    "Register"
                        .cstr_cs(visible_light(), CstrStyle::Heading)
                        .label(ui);
                    if Button::click("New Player").ui(ui).clicked() {
                        register_empty();
                        once_on_register_empty(|_, _, status| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                OperationsPlugin::add(|world| {
                                    Notification::new_string("New player created".to_owned())
                                        .push(world);
                                    let identity = ConnectOption::get(world).creds.identity.clone();
                                    let user = TUser::find(|u| u.identities.contains(&identity))
                                        .expect("Failed to find user after registration");
                                    world.resource_mut::<LoginData>().identity_user = Some(user);
                                })
                            }
                            spacetimedb_sdk::reducer::Status::Failed(e) => {
                                Notification::new_string(format!("Registration failed: {e}"))
                                    .error()
                                    .push_op()
                            }
                            _ => panic!(),
                        });
                    }
                    br(ui);
                    "Login".cstr_cs(visible_light(), CstrStyle::Heading).label(ui);
                    Input::new("name").ui_string(&mut ld.name_field, ui);
                    Input::new("password")
                        .password()
                        .ui_string(&mut ld.pass_field, ui);
                    if Button::click("Submit").ui(ui).clicked() {
                        login(ld.name_field.clone(), ld.pass_field.clone());
                        once_on_login(|_, _, status, name, _| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                let name = name.clone();
                                OperationsPlugin::add(move |world| {
                                    let user = TUser::find_by_name(name).unwrap();
                                    Self::complete(user, world);
                                });
                            }
                            spacetimedb_sdk::reducer::Status::Failed(e) => {
                                let text = format!("Login failed: {e}");
                                error!("{text}");
                                Notification::new_string(text).error().push_op();
                            }
                            _ => panic!(),
                        });
                    }
                }
            });
        });
    }
}
