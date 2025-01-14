use spacetimedb_sdk::Table;

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
            name_field: default(),
            pass_field: default(),
            identity_player: identity_user,
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
    }
    pub fn login_ui(ui: &mut Ui, world: &mut World) {
        center_window("login", ui.ctx(), |ui| {
            ui.vertical_centered_justified(|ui| {
                let mut ld = world.resource_mut::<LoginData>();
                if let Some(player) = ld.identity_player.clone() {
                    format!("Login as {}", player.name)
                        .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                        .label(ui);
                    if Button::new("Login").ui(ui).clicked() {
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
                        let _ = cn().reducers.register_empty();
                    }
                    br(ui);
                    "Login".cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading).label(ui);
                    Input::new("name").ui_string(&mut ld.name_field, ui);
                    Input::new("password")
                        .password()
                        .ui_string(&mut ld.pass_field, ui);
                    if Button::new("Submit").ui(ui).clicked() {
                        let _ = crate::login::login(
                            &cn().reducers,
                            ld.name_field.clone(),
                            ld.pass_field.clone(),
                        );
                    }
                }
            });
        });
    }
}
