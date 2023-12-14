use crate::module_bindings::{set_email, set_name, User};

use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::edit_state_ui.run_if(in_state(GameState::Profile)),
        )
        .add_systems(OnEnter(GameState::Profile), Self::on_enter);
    }
}

#[derive(Resource)]
struct ProfileEditData {
    user: User,
}

impl ProfilePlugin {
    fn on_enter(world: &mut World) {
        world.insert_resource(ProfileEditData {
            user: User::filter_by_identity(identity().unwrap()).expect("User not found"),
        });
    }

    pub fn ui(world: &mut World) {
        window("PROFILE").show(&egui_context(world), |ui| {
            if let Ok(identity) = identity() {
                let own_ladder_length = TableLadder::filter_by_creator(identity.clone())
                    .find(|l| l.status.eq(&module_bindings::LadderStatus::Fresh))
                    .map(|l| l.levels.len())
                    .unwrap_or_default();
                let beaten_ladders = TableLadder::filter_by_owner(identity.clone())
                    .filter(|l| l.status.eq(&module_bindings::LadderStatus::Beaten))
                    .collect_vec();
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

    fn edit_state_ui(world: &mut World) {
        let ctx = &egui_context(world);
        Window::new("Profile")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let user = &mut world.resource_mut::<ProfileEditData>().user;
                    ui.horizontal(|ui| {
                        ui.label("name:");
                        let mut name = user.name.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut name).changed() {
                            user.name = Some(name);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("email:");
                        let mut email = user.email.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut email).changed() {
                            user.email = Some(email);
                        }
                    });
                    if ui.button("Save").clicked() {
                        set_name(user.name.clone().unwrap_or_default());
                        set_email(user.email.clone().unwrap_or_default());
                        GameState::MainMenu.change(world);
                    }
                });
            });
    }
}
