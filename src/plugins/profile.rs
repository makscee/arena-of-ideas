use crate::module_bindings::{set_email, set_name, User};

use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui.run_if(in_state(GameState::Profile)))
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
            user: User::filter_by_identity(identity().unwrap()).expect("User not loaded"),
        });
    }

    fn ui(world: &mut World) {
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
