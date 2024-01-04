use crate::module_bindings::{
    once_on_set_name, once_on_set_password, set_name, set_password, User,
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
        if let Some(name) = LoginPlugin::get_username() {
            let user = User::filter_by_name(name).expect("User not found");
            world.insert_resource(ProfileEditData {
                user,
                old_pass: default(),
                pass: default(),
                pass_repeat: default(),
            });
        }
    }

    pub fn ui(world: &mut World) {
        window("PROFILE").show(&egui_context(world), |ui| {
            if let Some(username) = LoginPlugin::get_username() {
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
                        once_on_set_name(|_, _, status, name| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                LoginPlugin::save_current_user(name.clone());
                                AlertPlugin::add(
                                    None,
                                    "Name updated successfully".to_owned(),
                                    None,
                                );
                            }
                            spacetimedb_sdk::reducer::Status::Failed(e) => AlertPlugin::add_error(
                                Some("SET NAME ERROR".to_owned()),
                                e.clone(),
                                None,
                            ),
                            spacetimedb_sdk::reducer::Status::OutOfEnergy => panic!(),
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
                    ui.set_enabled(
                        !data.old_pass.is_empty()
                            && !data.pass.is_empty()
                            && data.pass.eq(&data.pass_repeat),
                    );
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
            }
        });
    }
}
