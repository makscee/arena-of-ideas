use crate::module_bindings::{
    logout, once_on_set_name, once_on_set_password, set_name, set_password, User,
};

use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Resource)]
struct ProfileEditData {
    user: User,
    old_pass: String,
    pass: String,
    pass_repeat: String,
}

impl ProfilePlugin {
    pub fn load(world: &mut World) {
        if let Some(data) = LoginPlugin::get_user_data() {
            let user = User::filter_by_name(data.name).expect("User not found");
            world.insert_resource(ProfileEditData {
                user,
                old_pass: default(),
                pass: default(),
                pass_repeat: default(),
            });
        }
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        window("PROFILE").show(ctx, |ui| {
            if let Some(data) = LoginPlugin::get_user_data() {
                let username = data.name;
                frame(ui, |ui| {
                    let user = &mut world.resource_mut::<ProfileEditData>().user;
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("name:");
                        });
                        ui.vertical(|ui| {
                            let mut name = user.name.clone();
                            if ui.text_edit_singleline(&mut name).changed() {
                                user.name = name;
                            }
                        });
                    });
                    ui.set_enabled(!user.name.eq(&username));
                    if ui.button("Save").clicked() {
                        set_name(user.name.clone());
                        once_on_set_name(move |_, _, status, name| {
                            debug!("set name callback");
                            match status {
                                spacetimedb_sdk::reducer::Status::Committed => {
                                    LoginPlugin::save_current_user(
                                        name.clone(),
                                        data.id,
                                        data.identity,
                                    );
                                    AlertPlugin::add(
                                        None,
                                        "Name updated successfully".to_owned(),
                                        None,
                                    );
                                }
                                spacetimedb_sdk::reducer::Status::Failed(e) => {
                                    AlertPlugin::add_error(
                                        Some("SET NAME ERROR".to_owned()),
                                        e.clone(),
                                        None,
                                    )
                                }
                                spacetimedb_sdk::reducer::Status::OutOfEnergy => panic!(),
                            };
                        });
                    }
                });
                frame(ui, |ui| {
                    let mut data = world.resource_mut::<ProfileEditData>();
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("old password:");
                            ui.label("new password:");
                            ui.label("repeat:");
                        });
                        ui.vertical(|ui| {
                            TextEdit::singleline(&mut data.old_pass)
                                .password(true)
                                .ui(ui);
                            TextEdit::singleline(&mut data.pass).password(true).ui(ui);
                            TextEdit::singleline(&mut data.pass_repeat)
                                .password(true)
                                .ui(ui);
                        });
                    });
                    ui.set_enabled(!data.pass.is_empty() && data.pass.eq(&data.pass_repeat));
                    if ui.button("Save").clicked() {
                        set_password(data.old_pass.clone(), data.pass.clone());
                        once_on_set_password(|_, _, status, _, _| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                AlertPlugin::add(
                                    None,
                                    "Password updated successfully".to_owned(),
                                    None,
                                );
                            }
                            spacetimedb_sdk::reducer::Status::Failed(e) => AlertPlugin::add_error(
                                Some("SET PASSWORD ERROR".to_owned()),
                                e.clone(),
                                None,
                            ),
                            spacetimedb_sdk::reducer::Status::OutOfEnergy => panic!(),
                        });
                    }
                });

                ui.collapsing("Login", |ui| {
                    LoginPlugin::login(ui, world);
                });

                frame(ui, |ui| {
                    if ui.button("LOGOUT").clicked() {
                        logout();
                        world.send_event(AppExit);
                    }
                });
            }
        });
    }
}
