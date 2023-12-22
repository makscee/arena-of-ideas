use bevy_egui::egui::epaint::{PathShape, RectShape};
use bevy_kira_audio::{AudioApp, AudioSource};

use super::*;

pub struct AudioPlugin;

#[derive(Resource)]
struct BackgroundChannel {
    handle: Handle<AudioInstance>,
}

#[derive(Resource, Clone)]
pub struct AudioData {
    pub play_delta: Option<f32>,
    pub prev_pos: Option<f32>,
    pub need_rate: f64,
    pub speed: f64,
    pub cur_rate: f64,
    pub background: Handle<AudioSource>,
    pub background_filtered: Handle<AudioSource>,
}
const RATE_CHANGE_SPEED: f64 = 2.0;

impl Default for AudioData {
    fn default() -> Self {
        Self {
            play_delta: default(),
            prev_pos: default(),
            need_rate: 1.0,
            speed: 1.0,
            cur_rate: default(),
            background: default(),
            background_filtered: default(),
        }
    }
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<BackgroundChannel>()
            .insert_resource(BackgroundChannel { handle: default() })
            .add_systems(Startup, Self::setup)
            .add_systems(OnEnter(GameState::Shop), Self::start_filtered_background)
            .add_systems(OnEnter(GameState::Battle), Self::start_normal_background)
            .add_systems(
                OnEnter(GameState::HeroGallery),
                Self::start_normal_background,
            )
            .add_systems(
                OnEnter(GameState::HeroEditor),
                Self::start_normal_background,
            )
            .add_systems(OnEnter(GameState::MainMenu), Self::stop_background)
            .add_systems(Update, Self::update)
            .add_systems(Update, Self::ui.run_if(in_state(GameState::Battle)));
    }
}

impl AudioPlugin {
    fn update(world: &mut World) {
        let mut data = world.resource::<AudioData>().clone();
        data.cur_rate += (data.need_rate * data.speed - data.cur_rate)
            * RATE_CHANGE_SPEED
            * world.resource::<Time>().delta_seconds_f64();
        Self::background_channel(world).set_playback_rate(data.cur_rate);

        let pos = Self::background_position(world);
        if let Some(pos) = pos {
            if let Some(prev_pos) = data.prev_pos {
                let delta = pos as f32 - prev_pos;
                if delta.abs() < 1.0 {
                    data.play_delta = Some(delta);
                } else {
                    data.play_delta = None;
                }
            } else {
                data.play_delta = None;
            }
            data.prev_pos = Some(pos as f32);
        }
        world.insert_resource(data);
    }

    fn setup(world: &mut World) {
        let bg: Handle<AudioSource> = world.resource::<AssetServer>().load("ron/audio/bg.ogg.ron");
        let bg_filtered: Handle<AudioSource> = world
            .resource::<AssetServer>()
            .load("ron/audio/bg_filtered.ogg.ron");

        let data = AudioData {
            background: bg,
            background_filtered: bg_filtered,
            ..default()
        };
        world.insert_resource(data);
    }

    fn background_channel(world: &World) -> &AudioChannel<BackgroundChannel> {
        world.resource::<AudioChannel<BackgroundChannel>>()
    }

    fn start_filtered_background(world: &mut World) {
        Self::start_background(true, world);
    }

    fn start_normal_background(world: &mut World) {
        Self::start_background(false, world);
    }

    fn start_background(filtered: bool, world: &mut World) {
        let audio = if filtered {
            world.resource::<AudioData>().background_filtered.clone()
        } else {
            world.resource::<AudioData>().background.clone()
        };
        let pos = Self::background_position(world).unwrap_or_default();
        Self::stop_background(world);
        let channel = Self::background_channel(world);
        let handle = channel.play(audio).looped().start_from(pos).handle();
        world.insert_resource(BackgroundChannel { handle });
    }

    fn stop_background(world: &World) {
        let channel = Self::background_channel(world);
        channel.stop();
    }

    pub fn background_position(world: &World) -> Option<f64> {
        let instance = &world.resource::<BackgroundChannel>().handle;
        let channel = world.resource::<AudioChannel<BackgroundChannel>>();

        channel.state(instance).position()
    }

    pub fn update_settings(settings: &SettingsData, world: &mut World) {
        let channel = world.resource::<AudioChannel<BackgroundChannel>>();
        channel.set_volume(settings.master_volume);
    }

    pub fn beat_timeframe() -> f32 {
        60.0 / 100.0
    }

    pub fn to_next_beat(world: &World) -> f32 {
        let timeframe = Self::beat_timeframe();
        let pos = Self::background_position(world).unwrap_or_default() as f32;
        (1.0 - (pos / timeframe).fract()) * timeframe
    }

    pub fn beat_index(world: &World) -> usize {
        let timeframe = Self::beat_timeframe();
        let pos = Self::background_position(world).unwrap_or_default() as f32;
        (pos / timeframe) as usize
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut data = world.resource::<AudioData>().clone();

        window("PLAYBACK")
            .anchor(Align2::CENTER_BOTTOM, [0.0, -50.0])
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                frame(ui, |ui| {
                    ui.columns(3, |ui| {
                        ui[0].vertical_centered_justified(|ui| {
                            AudioControls::Reverse.show(&mut data, world, ui);
                            ui.add_space(15.0);
                            AudioControls::SkipStart.show(&mut data, world, ui);
                        });
                        ui[1].vertical_centered_justified(|ui| {
                            AudioControls::Pause.show(&mut data, world, ui);
                        });
                        ui[2].vertical_centered_justified(|ui| {
                            AudioControls::Play.show(&mut data, world, ui);
                            ui.add_space(15.0);
                            AudioControls::SkipEnd.show(&mut data, world, ui);
                        });
                    });
                });
                frame(ui, |ui| {
                    ui.columns(3, |ui| {
                        ui[0].vertical_centered_justified(|ui| {
                            AudioControls::Speed1.show(&mut data, world, ui);
                        });
                        ui[1].vertical_centered_justified(|ui| {
                            AudioControls::Speed2.show(&mut data, world, ui);
                        });
                        ui[2].vertical_centered_justified(|ui| {
                            AudioControls::Speed3.show(&mut data, world, ui);
                        });
                    });
                    ui.label(format!("{:.2}", GameTimer::get(world).play_head()));
                });
            });
        world.insert_resource(data);
    }
}

enum AudioControls {
    Play,
    Reverse,
    Pause,
    StepForward,
    StepBackward,
    Speed1,
    Speed2,
    Speed3,
    SkipStart,
    SkipEnd,
}

impl AudioControls {
    fn show(self, data: &mut AudioData, world: &mut World, ui: &mut Ui) {
        match &self {
            AudioControls::Speed1 => {
                let active = data.speed == 1.0;
                let text = "x1";
                if if active {
                    ui.button_primary(text)
                } else {
                    ui.button(text)
                }
                .clicked()
                {
                    data.speed = 1.0;
                }
                return;
            }
            AudioControls::Speed2 => {
                let active = data.speed == 2.0;
                let text = "x2";
                if if active {
                    ui.button_primary(text)
                } else {
                    ui.button(text)
                }
                .clicked()
                {
                    data.speed = 2.0;
                }
                return;
            }
            AudioControls::Speed3 => {
                let active = data.speed == 4.0;
                let text = "x4";
                if if active {
                    ui.button_primary(text)
                } else {
                    ui.button(text)
                }
                .clicked()
                {
                    data.speed = 4.0;
                }
                return;
            }
            _ => {}
        }

        let size = match &self {
            AudioControls::Play
            | AudioControls::Reverse
            | AudioControls::Pause
            | AudioControls::Speed1
            | AudioControls::Speed2
            | AudioControls::Speed3 => egui::vec2(30.0, 30.0),
            AudioControls::SkipStart | AudioControls::SkipEnd => egui::vec2(75.0, 30.0) * 0.5,
            AudioControls::StepForward => todo!(),
            AudioControls::StepBackward => todo!(),
        };
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
        let mut clicked = false;
        if response.clicked() {
            clicked = true;
            response.mark_changed();
        }
        response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, ""));
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        match self {
            AudioControls::Play => {
                if clicked {
                    data.need_rate = 1.0;
                }
                let active = data.need_rate > 0.0;
                let points = [rect.left_top(), rect.right_center(), rect.left_bottom()];
                ui.painter().add(egui::Shape::Path(if active {
                    PathShape::convex_polygon(points.into(), visuals.bg_fill, visuals.bg_stroke)
                } else {
                    PathShape::closed_line(points.into(), visuals.fg_stroke)
                }));
            }
            AudioControls::Reverse => {
                if clicked {
                    data.need_rate = -1.0;
                }
                let active = data.need_rate < 0.0;
                let points = [rect.right_top(), rect.left_center(), rect.right_bottom()];
                ui.painter().add(egui::Shape::Path(if active {
                    PathShape::convex_polygon(points.into(), visuals.bg_fill, visuals.bg_stroke)
                } else {
                    PathShape::closed_line(points.into(), visuals.fg_stroke)
                }));
            }
            AudioControls::Pause => {
                if clicked {
                    data.need_rate = 0.0;
                }
                let active = data.need_rate == 0.0;
                let mut rect1 = rect;
                *rect1.right_mut() -= rect.width() * 0.6;
                let mut rect2 = rect;
                *rect2.left_mut() += rect.width() * 0.6;

                ui.painter().add(if active {
                    Vec::from([
                        egui::Shape::Rect(RectShape::filled(
                            rect1,
                            Rounding::ZERO,
                            visuals.bg_fill,
                        )),
                        egui::Shape::Rect(RectShape::filled(
                            rect2,
                            Rounding::ZERO,
                            visuals.bg_fill,
                        )),
                    ])
                } else {
                    Vec::from([
                        egui::Shape::Rect(RectShape::stroke(
                            rect1,
                            Rounding::ZERO,
                            visuals.fg_stroke,
                        )),
                        egui::Shape::Rect(RectShape::stroke(
                            rect2,
                            Rounding::ZERO,
                            visuals.fg_stroke,
                        )),
                    ])
                });
            }
            AudioControls::StepForward => todo!(),
            AudioControls::StepBackward => todo!(),
            AudioControls::SkipStart => {
                if clicked {
                    GameTimer::get_mut(world).play_head_to(0.0);
                }
                let mut rect1 = rect;
                *rect1.right_mut() -= rect.width() * 0.8;
                let mut rect2 = rect.translate(egui::vec2(rect1.width(), 0.0));
                *rect2.right_mut() -= rect.width() * 0.6;
                let mut rect3 = rect;
                *rect3.left_mut() += rect.width() * 0.6;

                ui.painter().add(Vec::from([
                    egui::Shape::Rect(RectShape::stroke(rect1, Rounding::ZERO, visuals.fg_stroke)),
                    egui::Shape::Path(PathShape::closed_line(
                        [rect2.right_top(), rect2.left_center(), rect2.right_bottom()].into(),
                        visuals.fg_stroke,
                    )),
                    egui::Shape::Path(PathShape::closed_line(
                        [rect3.right_top(), rect3.left_center(), rect3.right_bottom()].into(),
                        visuals.fg_stroke,
                    )),
                ]));
            }
            AudioControls::SkipEnd => {
                if clicked {
                    GameTimer::get_mut(world).skip_to_end();
                }
                let mut rect1 = rect;
                *rect1.right_mut() -= rect.width() * 0.6;
                let rect2 = rect1.translate(egui::vec2(rect1.width(), 0.0));
                let mut rect3 = rect;
                *rect3.left_mut() += rect.width() - rect.width() / 5.0;

                ui.painter().add(Vec::from([
                    egui::Shape::Path(PathShape::closed_line(
                        [rect1.left_top(), rect1.right_center(), rect1.left_bottom()].into(),
                        visuals.fg_stroke,
                    )),
                    egui::Shape::Path(PathShape::closed_line(
                        [rect2.left_top(), rect2.right_center(), rect2.left_bottom()].into(),
                        visuals.fg_stroke,
                    )),
                    egui::Shape::Rect(RectShape::stroke(rect3, Rounding::ZERO, visuals.fg_stroke)),
                ]));
            }
            _ => {}
        };
    }
}
