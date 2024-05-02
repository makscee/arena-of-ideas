use crate::module_bindings::{
    logout, once_on_set_name, once_on_set_password, set_name, set_password, User,
};

use self::module_bindings::ArenaArchive;

use super::*;

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenProfiles>().add_systems(
            Update,
            (Self::ui, Self::close_all.run_if(state_changed::<GameState>)),
        );
    }
}

#[derive(Resource)]
struct ProfileEditData {
    user: User,
    old_pass: String,
    pass: String,
    pass_repeat: String,
}

#[derive(Resource, Default)]
struct OpenProfiles(HashMap<u64, ProfileViewData>);

#[derive(Default)]
struct ProfileViewData {
    max_round: u32,
    total_runs: u32,
    total_wins: u32,
    total_loses: u32,
    win_rate: f32,
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

    pub fn open_player_profile(user_id: u64, world: &mut World) {
        let mut data = ProfileViewData::default();
        for run in ArenaArchive::filter_by_user_id(user_id) {
            data.total_wins += run.wins;
            data.total_loses += run.loses;
            data.total_runs += 1;
            data.max_round = data.max_round.max(run.round);
        }
        if data.total_wins > 0 || data.total_loses > 0 {
            data.win_rate = data.total_wins as f32 / (data.total_loses + data.total_wins) as f32;
        }
        world.resource_mut::<OpenProfiles>().0.insert(user_id, data);
    }
    pub fn close_player_profile(user_id: u64, world: &mut World) {
        world.resource_mut::<OpenProfiles>().0.remove(&user_id);
    }
    pub fn close_all(world: &mut World) {
        world.resource_mut::<OpenProfiles>().0.clear();
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        for (user_id, data) in &world.resource::<OpenProfiles>().0 {
            let user_id = *user_id;
            let user = if let Some(user) = User::filter_by_id(user_id) {
                user
            } else {
                continue;
            };
            window("PLAYER PROFILE")
                .id(user_id)
                .set_width(400.0)
                .default_pos(ctx.screen_rect().center())
                .set_close_action(Box::new(move |world| {
                    Self::close_player_profile(user_id, world)
                }))
                .show(ctx, |ui| {
                    frame(ui, |ui| {
                        user.name.add_color(white()).label(ui);
                        if user.online {
                            "online".add_color(white()).label(ui);
                        } else {
                            "offline".add_color(light_gray()).label(ui);
                            text_dots_text(
                                &"last online:".to_colored(),
                                &format_timestamp(user.last_login).add_color(white()),
                                ui,
                            );
                        }
                        ui.add_space(10.0);
                        text_dots_text(
                            &"runs played".to_colored(),
                            &data.total_runs.to_string().add_color(white()),
                            ui,
                        );
                        text_dots_text(
                            &"max round reached".to_colored(),
                            &data.max_round.to_string().add_color(white()),
                            ui,
                        );
                        text_dots_text(
                            &"total wins".to_colored(),
                            &data.total_wins.to_string().add_color(white()),
                            ui,
                        );
                        text_dots_text(
                            &"total loses".to_colored(),
                            &data.total_loses.to_string().add_color(white()),
                            ui,
                        );
                        text_dots_text(
                            &"win rate".to_colored(),
                            &data.win_rate.to_string().add_color(white()),
                            ui,
                        );
                    })
                });
        }
    }

    pub fn edit_ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        window("EDIT PROFILE").set_width(350.0).show(ctx, |ui| {
            if let Some(data) = LoginPlugin::get_user_data() {
                frame(ui, |ui| {
                    if ui.button("VIEW PLAYER PROFILE").clicked() {
                        Self::open_player_profile(data.id, world);
                    }
                });
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
                    ui.set_enabled(!user.name.eq(&data.name));
                    if ui.button("Save").clicked() {
                        set_name(user.name.clone());
                        once_on_set_name(move |_, _, status, name| {
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
                ui.collapsing("Password", |ui| {
                    frame(ui, |ui| {
                        let mut data = world.resource_mut::<ProfileEditData>();
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                if data.user.pass_hash.is_some() {
                                    ui.label("old password:");
                                }
                                ui.label("new password:");
                                ui.label("repeat:");
                            });
                            ui.vertical(|ui| {
                                if data.user.pass_hash.is_some() {
                                    TextEdit::singleline(&mut data.old_pass)
                                        .password(true)
                                        .ui(ui);
                                }
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
                                spacetimedb_sdk::reducer::Status::Failed(e) => {
                                    AlertPlugin::add_error(
                                        Some("SET PASSWORD ERROR".to_owned()),
                                        e.clone(),
                                        None,
                                    )
                                }
                                spacetimedb_sdk::reducer::Status::OutOfEnergy => panic!(),
                            });
                        }
                    });
                });

                ui.collapsing("Login", |ui| {
                    LoginPlugin::login(ui, world);
                });

                frame(ui, |ui| {
                    let text =
                        "You will only be able to log back in with password (if set).\nContinue?"
                            .to_owned();
                    if ui.button_red("LOGOUT & EXIT").clicked() {
                        AlertPlugin::add(
                            Some("LOGOUT?".to_owned()),
                            text,
                            Some(Box::new(|w| {
                                logout();
                                w.send_event(AppExit);
                            })),
                        );
                    }
                });
            }
        });
    }
}
