use crate::module_bindings::{once_on_set_name, set_name, User};

use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Resource)]
struct ProfileEditData {
    user: User,
}

impl ProfilePlugin {
    pub fn load(world: &mut World) {
        if let Some(name) = LoginPlugin::get_username() {
            let user = User::filter_by_name(name).expect("User not found");
            world.insert_resource(ProfileEditData { user });
        }
    }

    pub fn ui(world: &mut World) {
        window("PROFILE").show(&egui_context(world), |ui| {
            if LoginPlugin::get_username().is_some() {
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
                    if ui.button("Save").clicked() {
                        set_name(user.name.clone());
                        once_on_set_name(|_, _, status, name| match status {
                            spacetimedb_sdk::reducer::Status::Committed => {
                                LoginPlugin::save_current_user(name.clone())
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
            }
        });
    }
}
