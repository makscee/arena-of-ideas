use super::*;

#[derive(Resource, Default)]
pub struct UserName(String);

#[derive(Resource, Default)]
pub struct Password(String);

#[allow(dead_code)]
pub fn ui_example_system(
    mut contexts: EguiContexts,
    mut name: ResMut<UserName>,
    mut password: ResMut<Password>,
) {
    let name = &mut name.0;
    let password = &mut password.0;
    CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.horizontal_centered(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Arena of Ideas");
                ui.add_space(20.0);
                ui.label("Login");
                ui.add(TextEdit::singleline(name).desired_width(150.0));
                ui.label("Password");
                ui.add(
                    TextEdit::singleline(password)
                        .desired_width(150.0)
                        .password(true),
                );
                if ui.button("Submit").clicked() {
                    debug!("Login: {name} {password}");
                }
            })
        })
    });
}
