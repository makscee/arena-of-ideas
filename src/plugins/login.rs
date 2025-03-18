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
        cn().reducers.on_login_by_identity(|e| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                OperationsPlugin::add(|world| {
                    world.resource_mut::<LoginData>().login_requested = false;
                    let mut cs = client_state().clone();
                    cs.last_logged_in = None;
                    cs.save_to_cache();
                });
            });
        });
        cn().reducers.on_login(|e, name, _| {
            if !e.check_identity() {
                return;
            }
            e.event.on_success_error(LoginPlugin::complete, || {
                OperationsPlugin::add(|world| {
                    world.resource_mut::<LoginData>().login_requested = false;
                    let mut cs = client_state().clone();
                    cs.last_logged_in = None;
                    cs.save_to_cache();
                });
            });
        });
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
            for n in cn().db.nodes_world().iter() {
                dbg!(n);
            }
            let Some(identity_node) = PlayerIdentity::find_by_data(Some(identity.to_string()))
            else {
                "Failed to find Player after login".notify_error(world);
                return;
            };
            dbg!(&identity_node);
            if let Some(player) = Player::load(identity_node.parent.unwrap()) {
                dbg!(&player);
                let mut cs = client_state().clone();
                cs.last_logged_in = Some((player.name.clone(), identity));
                cs.save();
                LoginOption { player }.save(world);
            } else {
                error!("Failed to find Player");
            }
            GameState::proceed(world);
        });
    }
    pub fn pane_register(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            "New Player"
                .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Heading2)
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
    pub fn pane_login(ui: &mut Ui, world: &mut World) {
        ui.vertical_centered_justified(|ui| {
            let mut ld = world.resource_mut::<LoginData>();
            ui.add_space(ui.available_height() * 0.3);
            ui.set_width(350.0.at_most(ui.available_width()));
            let mut cs = client_state().clone();
            if let Some((name, identity)) = &cs.last_logged_in {
                format!("Login as {name}")
                    .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Heading2)
                    .label(ui);
                if (client_settings().auto_login && !ld.login_requested)
                    || Button::new("Login")
                        .enabled(!ld.login_requested)
                        .ui(ui)
                        .clicked()
                {
                    ld.login_requested = true;
                    let mut cs = client_settings().clone();
                    cs.auto_login = false;
                    cs.save_to_cache();
                    let _ = cn().reducers.login_by_identity();
                }
                br(ui);
                if Button::new("Logout")
                    .enabled(!ld.login_requested)
                    .gray(ui)
                    .ui(ui)
                    .clicked()
                    || ConnectOption::get(world).identity != *identity
                {
                    cs.last_logged_in = None;
                    cs.save_to_cache();
                }
            } else {
                let mut ld = world.resource_mut::<LoginData>();
                "Login"
                    .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Heading)
                    .label(ui);
                Input::new("name").ui_string(&mut ld.name_field, ui);
                Input::new("password")
                    .password()
                    .ui_string(&mut ld.pass_field, ui);
                if Button::new("Submit")
                    .enabled(!ld.login_requested)
                    .ui(ui)
                    .clicked()
                {
                    ld.login_requested = true;
                    cn().reducers
                        .login(ld.name_field.clone(), ld.pass_field.clone())
                        .unwrap();
                }
                space(ui);
                br(ui);
                if Button::new("New Player")
                    .enabled(!ld.login_requested)
                    .ui(ui)
                    .clicked()
                {
                    GameState::Register.set_next(world);
                }
            }
        });
    }
}
