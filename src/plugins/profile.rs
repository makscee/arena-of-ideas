use crate::module_bindings::{set_email, set_name, LadderStatus, User};

use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource)]
struct ProfileEditData {
    user: User,
}

impl ProfilePlugin {
    pub fn load(world: &mut World) {
        let user = User::filter_by_identity(identity().unwrap()).expect("User not found");
        world.insert_resource(ProfileEditData { user });
    }

    pub fn ui(world: &mut World) {
        window("PROFILE").show(&egui_context(world), |ui| {
            if let Ok(identity) = identity() {
                let own_ladder_length = TableLadder::filter_by_creator(identity.clone())
                    .find(|l| matches!(l.status, LadderStatus::Fresh(..)))
                    .map(|l| l.levels.len())
                    .unwrap_or_default();
                let beaten_ladders = TableLadder::filter_by_owner(identity.clone())
                    .filter(|l| matches!(l.status, LadderStatus::Beaten(..)))
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
                        &"Own ladder length".to_colored(),
                        &own_ladder_length.to_string().add_color(white()),
                        ui,
                    );
                    text_dots_text(
                        &"Beaten ladders count".to_colored(),
                        &beaten_ladders.len().to_string().add_color(white()),
                        ui,
                    );
                    for (i, ladder) in beaten_ladders.into_iter().enumerate() {
                        ui.collapsing(format!("{}. {} levels", i + 1, ladder.levels.len()), |ui| {
                            for level in ladder.levels.iter() {
                                ui.label(PackedTeam::from_ladder_string(level, world).to_string());
                            }
                        });
                    }
                });
            }
        });
    }
}
