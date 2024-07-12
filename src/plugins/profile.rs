use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Profile), Self::on_enter);
    }
}

#[derive(Resource)]
struct ProfileEditData {
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
    fn update_user(world: &mut World) {
        LoginOption {
            user: TUser::filter_by_id(user_id()).unwrap(),
        }
        .save(world);
    }
    pub fn settings_ui(ui: &mut Ui, world: &mut World) {
        let user = &LoginOption::get(world).user;
        let has_pass = user.pass_hash.is_some();
        let mut ped = world.resource_mut::<ProfileEditData>();
        Input::new("name").ui(&mut ped.name, ui);
        if Button::click("Submit".into())
            .enabled(!ped.name.eq(user_name()))
            .ui(ui)
            .clicked()
        {
            set_name(ped.name.clone());
            once_on_set_name(|_, _, status, name| {
                let name = name.clone();
                match status {
                    spacetimedb_sdk::reducer::Status::Committed => {
                        OperationsPlugin::add(move |world| {
                            Notification::new(format!("Name successfully changed to {name}"))
                                .push(world);
                            Self::update_user(world);
                        })
                    }
                    spacetimedb_sdk::reducer::Status::Failed(e) => {
                        Notification::new(format!("Name change error: {e}"))
                            .error()
                            .push_op()
                    }
                    _ => panic!(),
                }
            });
        };
        br(ui);
        if has_pass {
            Input::new("old password")
                .password()
                .ui(&mut ped.old_pass, ui);
        }
        Input::new("new password").password().ui(&mut ped.pass, ui);
        Input::new("new password repeat")
            .password()
            .ui(&mut ped.pass_repeat, ui);
        if Button::click("Submit".into())
            .enabled(!ped.pass.is_empty() && ped.pass.eq(&ped.pass_repeat))
            .ui(ui)
            .clicked()
        {
            set_password(ped.old_pass.clone(), ped.pass.clone());
            once_on_set_password(|_, _, status, _, _| match status {
                spacetimedb_sdk::reducer::Status::Committed => {
                    OperationsPlugin::add(|world| {
                        Notification::new("Password updated successfully".to_owned()).push(world);
                        Self::update_user(world);
                        let mut ped = world.resource_mut::<ProfileEditData>();
                        ped.pass.clear();
                        ped.pass_repeat.clear();
                        ped.old_pass.clear();
                    });
                }
                spacetimedb_sdk::reducer::Status::Failed(e) => {
                    Notification::new(format!("Password change error: {e}"))
                        .error()
                        .push_op()
                }
                _ => panic!(),
            });
        }
        br(ui);
    }
}
