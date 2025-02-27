use spacetimedb_lib::Identity;
use spacetimedb_sdk::{DbContext, Table};

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
    login_requested: bool,
}

impl LoginPlugin {
    fn login(world: &mut World) {
        let co = ConnectOption::get(world);
        if currently_fulfilling() == GameOption::ForceLogin {
            // if let Some((name, i)) = client_state().last_logged_in {
            // Self::complete(Some(player.clone()), world);
            // }
        }
    }
    fn complete() {
        subscribe_game(Self::on_subscribed);
        subscribe_reducers();
    }
    fn on_subscribed() {
        OperationsPlugin::add(|world| {
            let identity = ConnectOption::get(world).identity;
            let player = cn()
                .db
                .tnodes()
                .data()
                .find(&PlayerIdentity::new(Some(identity.to_string())).get_data())
                .and_then(|d| Player::get(d.id));
            if let Some(player) = player {
                LoginOption { player }.save(world);
            } else {
                error!("Failed to find Player");
            }
            GameState::proceed(world);
        });
    }
    pub fn tab_register(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            "New Player"
                .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                .label(ui);
            let mut ld = world.resource_mut::<LoginData>();
            Input::new("name").ui_string(&mut ld.name_field, ui);
            Input::new("password")
                .password()
                .ui_string(&mut ld.pass_field, ui);
            if Button::new("Submit").ui(ui).clicked() {
                cn().reducers.on_register(|e, _, _| {
                    if !e.check_identity() {
                        return;
                    }
                    e.event.on_success(LoginPlugin::complete);
                });
                cn().reducers
                    .register(ld.name_field.clone(), ld.pass_field.clone())
                    .unwrap();
            }
        });
    }
    pub fn tab_login(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let mut ld = world.resource_mut::<LoginData>();
            if let Some((name, identity)) = &client_state().last_logged_in {
                format!("Login as {}", name)
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
                        e.event.on_success(LoginPlugin::complete);
                    });
                    let _ = cn().reducers.login_by_identity();
                }
                br(ui);
                if Button::new("Logout").gray(ui).ui(ui).clicked() {
                    todo!();
                }
            } else {
                let mut ld = world.resource_mut::<LoginData>();
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
                        e.event.on_success(LoginPlugin::complete);
                    });
                    cn().reducers
                        .login(ld.name_field.clone(), ld.pass_field.clone())
                        .unwrap();
                }
                br(ui);
                if "New Player"
                    .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)
                    .button(ui)
                    .clicked()
                {
                    GameState::Register.set_next(world);
                }
            }
        });
    }
}
