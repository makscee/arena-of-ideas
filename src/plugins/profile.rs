use crate::module_bindings::{set_email, set_name, TowerStatus, User};

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
        if let Ok(identity) = identity() {
            let user = User::filter_by_identity(identity).expect("User not found");
            world.insert_resource(ProfileEditData { user });
        }
    }

    pub fn ui(world: &mut World) {
        window("PROFILE").show(&egui_context(world), |ui| {
            if let Ok(identity) = identity() {
                let own_tower_length = TableTower::filter_by_creator(identity.clone())
                    .find(|l| matches!(l.status, TowerStatus::Fresh(..)))
                    .map(|l| l.levels.len())
                    .unwrap_or_default();
                let beaten_towers = TableTower::filter_by_owner(identity.clone())
                    .filter(|l| matches!(l.status, TowerStatus::Beaten(..)))
                    .collect_vec();
                frame(ui, |ui| {
                    let user = &mut world.resource_mut::<ProfileEditData>().user;
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("name:");
                            ui.label("email:");
                        });
                        ui.vertical(|ui| {
                            let mut name = user.name.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut name).changed() {
                                user.name = Some(name);
                            }
                            let mut email = user.email.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut email).changed() {
                                user.email = Some(email);
                            }
                        });
                    });
                    if ui.button("Save").clicked() {
                        set_name(user.name.clone().unwrap_or_default());
                        set_email(user.email.clone().unwrap_or_default());
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
                frame(ui, |ui| {
                    ui.set_enabled(world.resource::<CurrentCredentials>().creds.is_some());
                    let visuals = &mut ui.style_mut().visuals.widgets.inactive;
                    visuals.fg_stroke.color = red();
                    visuals.bg_stroke.color = red();
                    if ui.button("CLEAR IDENTITY").clicked() {
                        LoginPlugin::clear_saved_credentials(world);
                        Save::clear(world).unwrap();
                        world.send_event(AppExit);
                    }
                });
            }
        });
    }
}
