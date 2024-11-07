use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Profile), Self::on_enter);
    }
}

#[derive(Resource, Default)]
pub struct ProfileEditData {
    name: String,
    old_pass: String,
    pass: String,
    pass_repeat: String,
}

impl ProfilePlugin {
    fn on_enter(world: &mut World) {
        world.insert_resource(ProfileEditData {
            name: user_name().to_owned(),
            old_pass: default(),
            pass: default(),
            pass_repeat: default(),
        })
    }
    pub fn update_user(world: &mut World) {
        LoginOption {
            player: cn().db.player().id().find(&player_id()).unwrap(),
        }
        .save(world);
    }
    pub fn clear_edit(world: &mut World) {
        let mut ped = world.resource_mut::<ProfileEditData>();
        ped.pass.clear();
        ped.pass_repeat.clear();
        ped.old_pass.clear();
    }
    pub fn add_tile_settings(world: &mut World) {
        world.insert_resource(ProfileEditData {
            name: user_name().into(),
            old_pass: default(),
            pass: default(),
            pass_repeat: default(),
        });
        Tile::new(Side::Left, |ui, world| {
            world.resource_scope(|world, mut ped: Mut<ProfileEditData>| {
                Self::settings_ui(&mut ped, ui, world);
            })
        })
        .with_id("Profile Settings".into())
        .min_space(egui::vec2(200.0, 0.0))
        .push(world);
    }
    fn settings_ui(ped: &mut ProfileEditData, ui: &mut Ui, world: &mut World) {
        let player = &LoginOption::get(world).player;
        let has_pass = player.pass_hash.is_some();
        Input::new("name").ui_string(&mut ped.name, ui);
        if Button::click("Submit")
            .enabled(!ped.name.eq(user_name()))
            .ui(ui)
            .clicked()
        {
            cn().reducers.set_name(ped.name.clone());
        };
        br(ui);
        if has_pass {
            Input::new("old password")
                .password()
                .ui_string(&mut ped.old_pass, ui);
        }
        Input::new("new password")
            .password()
            .ui_string(&mut ped.pass, ui);
        Input::new("new password repeat")
            .password()
            .ui_string(&mut ped.pass_repeat, ui);
        if Button::click("Submit")
            .enabled(!ped.pass.is_empty() && ped.pass.eq(&ped.pass_repeat))
            .ui(ui)
            .clicked()
        {
            cn().reducers
                .set_password(ped.old_pass.clone(), ped.pass.clone());
        }
        br(ui);
    }
}
