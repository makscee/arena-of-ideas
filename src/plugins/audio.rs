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
            .add_systems(OnEnter(GameState::MainMenu), Self::stop_background)
            .add_systems(Update, Self::update)
            .add_systems(Update, Self::ui.run_if(in_state(GameState::Battle)));
    }
}

impl AudioPlugin {
    fn update(world: &mut World) {
        let mut data = world.resource::<AudioData>().clone();
        data.cur_rate += (data.need_rate - data.cur_rate)
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
        let input = world.resource::<Input<KeyCode>>().clone();
        let mut data = world.resource::<AudioData>().clone();
        Window::new("Playback")
            .anchor(Align2::CENTER_BOTTOM, [0.0, -50.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let skip_start = ui.button("|<<");
                    if skip_start.clicked() {
                        let t = -Self::to_next_beat(world);
                        GameTimer::get_mut(world).play_head_to(t);
                    }
                    let left = ui.button("<<");
                    if left.clicked() {
                        left.request_focus();
                    }
                    let pause = ui.button("||");
                    if pause.clicked() {
                        pause.request_focus();
                    }
                    let right = ui.button(">>");
                    if right.clicked() {
                        right.request_focus();
                    }
                    let skip_end = ui.button(">>|");
                    if skip_end.clicked() {
                        GameTimer::get_mut(world).skip_to_end();
                    }
                    if left.has_focus() || input.pressed(KeyCode::Left) {
                        data.need_rate = -2.0;
                    } else if right.has_focus() || input.pressed(KeyCode::Right) {
                        data.need_rate = 2.0;
                    } else if pause.has_focus() || input.pressed(KeyCode::Space) {
                        data.need_rate = 0.0;
                    } else {
                        data.need_rate = 1.0;
                    }
                })
            });
        world.insert_resource(data);
    }
}
