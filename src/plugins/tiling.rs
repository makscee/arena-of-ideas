use egui::ImageButton;

use super::*;

#[derive(Resource, Default)]
struct Data {
    t: f32,
}

pub struct TilingPlugin;

impl Plugin for TilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .add_systems(Startup, Self::setup)
            .init_resource::<Data>();
    }
}

impl TilingPlugin {
    fn setup(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        ctx.flip_name_enabled("Playback");
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        if just_pressed(KeyCode::Escape, world) {
            ctx.flip_name_enabled("Main Menu");
        }
        TopBottomPanel::top("top")
            .frame(Frame::none().inner_margin(Margin::same(4.0)))
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if let Some(fps) = world
                        .resource::<DiagnosticsStore>()
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                    {
                        if let Some(fps) = fps.smoothed() {
                            ui.label(format!("fps: {fps:.0}"));
                        }
                    }
                    ui.label(format!("arena-of-ideas {VERSION}"));
                })
            });
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                Tile::right("Main Menu")
                    .title()
                    .close_btn()
                    .content(|ui, _| {
                        Button::toggle_child("New Game").ui(ui);
                        Button::toggle_child("Settings").ui(ui);
                    })
                    .child(|ui, world| {
                        Tile::right("New Game")
                            .title()
                            .close_btn()
                            .content(|ui, _| {
                                if ui.button("test").clicked() {
                                    debug!("test");
                                }
                            })
                            .ui(ui, world);
                        Tile::right("Settings")
                            .title()
                            .close_btn()
                            .content(|ui, world| {
                                if ui.button("setting 1").clicked() {
                                    debug!("Test click");
                                }
                                br(ui);
                                if ui.button("setting 2").clicked() {
                                    debug!("Test click");
                                }
                                br(ui);
                            })
                            .ui(ui, world);
                    })
                    .ui(ui, world);
                if matches!(cur_state(world), GameState::Battle) {
                    Tile::bottom("Playback")
                        .transparent()
                        .content(|ui, world| {
                            ui.vertical_centered(|ui| {
                                let mut gt = GameTimer::get();
                                if ImageButton::new(if gt.paused() {
                                    Icon::Pause.image()
                                } else {
                                    Icon::Play.image()
                                })
                                .ui(ui)
                                .clicked()
                                {
                                    let paused = gt.paused();
                                    gt.pause(!paused);
                                }
                            });

                            Middle3::default().ui(
                                ui,
                                world,
                                |ui, world| {
                                    format!("{:.2}", GameTimer::get().play_head())
                                        .cstr_cs(WHITE, CstrStyle::Heading)
                                        .label(ui);
                                },
                                |ui, world| {
                                    const FF_LEFT_KEY: &str = "ff_back_btn";
                                    let pressed = get_context_bool(world, FF_LEFT_KEY);
                                    if pressed {
                                        GameTimer::get().advance_play(-delta_time(world) * 2.0);
                                    }
                                    let resp = ImageButton::new(Icon::FFBack.image())
                                        .tint(if pressed { YELLOW } else { WHITE })
                                        .ui(ui);
                                    set_context_bool(
                                        world,
                                        FF_LEFT_KEY,
                                        resp.contains_pointer() && left_mouse_pressed(world),
                                    );
                                },
                                |ui, world| {
                                    const FF_RIGHT_KEY: &str = "ff_forward_btn";
                                    let pressed = get_context_bool(world, FF_RIGHT_KEY);
                                    if pressed {
                                        GameTimer::get().advance_play(delta_time(world));
                                    }
                                    let resp = ImageButton::new(Icon::FFForward.image())
                                        .tint(if pressed { YELLOW } else { WHITE })
                                        .ui(ui);
                                    set_context_bool(
                                        world,
                                        FF_RIGHT_KEY,
                                        resp.contains_pointer() && left_mouse_pressed(world),
                                    );
                                },
                            );
                            Middle3::default().width(400.0).ui(
                                ui,
                                world,
                                |ui, world| {
                                    Slider::new("Playback Speed")
                                        .log()
                                        .name(false)
                                        .range(-20.0..=20.0)
                                        .ui(&mut GameTimer::get().playback_speed, ui);
                                },
                                |ui, _| {
                                    if ImageButton::new(Icon::SkipBack.image()).ui(ui).clicked() {
                                        GameTimer::get().play_head_to(0.0);
                                    }
                                },
                                |ui, _| {
                                    if ImageButton::new(Icon::SkipForward.image()).ui(ui).clicked()
                                    {
                                        GameTimer::get().skip_to_end();
                                    }
                                },
                            );
                        })
                        .ui(ui, world);
                }
            });
    }
}
