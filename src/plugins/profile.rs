use crate::module_bindings::{once_on_set_name, set_name, TowerStatus, User};

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
            if let Some(name) = LoginPlugin::get_username() {
                let own_tower_length = TableTower::filter_by_creator(name.to_owned())
                    .find(|l| matches!(l.status, TowerStatus::Fresh(..)))
                    .map(|l| l.levels.len())
                    .unwrap_or_default();
                let beaten_towers = TableTower::filter_by_owner(name.to_owned())
                    .filter(|l| matches!(l.status, TowerStatus::Beaten(..)))
                    .collect_vec();
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
                frame(ui, |ui| {
                    text_dots_text(
                        &"Own tower length".to_colored(),
                        &own_tower_length.to_string().add_color(white()),
                        ui,
                    );
                    text_dots_text(
                        &"Beaten towers count".to_colored(),
                        &beaten_towers.len().to_string().add_color(white()),
                        ui,
                    );
                    for (i, tower) in beaten_towers.into_iter().enumerate() {
                        ui.collapsing(format!("{}. {} levels", i + 1, tower.levels.len()), |ui| {
                            for level in tower.levels.iter() {
                                ui.label(PackedTeam::from_tower_string(level, world).to_string());
                            }
                        });
                    }
                });
            }
        });
    }
}
