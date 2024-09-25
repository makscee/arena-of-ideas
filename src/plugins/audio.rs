use bevy::audio::{AudioSink, AudioSinkPlayback, Volume};
use rand::seq::SliceRandom;

use super::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), Self::loaded)
            .add_systems(Update, Self::update);
    }
}

#[derive(Resource, AssetCollection)]
pub struct AudioAssets {
    #[asset(key = "audio_bg", collection(typed))]
    audio_bg: Vec<Handle<AudioSource>>,
    #[asset(key = "audio_click")]
    audio_click: Handle<AudioSource>,
    #[asset(key = "audio_coin")]
    audio_coin: Handle<AudioSource>,
    #[asset(key = "audio_start_game")]
    audio_start_game: Handle<AudioSource>,
}

#[derive(Resource, Debug)]
struct AudioResource {
    bg_inds: Vec<usize>,
    bg_cur: usize,
}

#[derive(Component)]
struct BackgroundAudioMarker;

pub enum SoundEffect {
    Click,
    Coin,
    StartGame,
}

static FX_QUEUE: Mutex<Vec<SoundEffect>> = Mutex::new(Vec::new());

impl AudioPlugin {
    pub fn set_music_volume(value: f32, world: &mut World) {
        if let Some((_, s)) = Self::music_sink(world) {
            s.set_volume(value);
        }
    }
    pub fn queue_sound(se: SoundEffect) {
        FX_QUEUE.lock().unwrap().push(se);
    }
    fn play(source: Handle<AudioSource>, world: &mut World) {
        world.spawn(AudioBundle {
            source,
            settings: PlaybackSettings::DESPAWN,
        });
    }
    fn music_sink(world: &mut World) -> Option<(Entity, &AudioSink)> {
        world
            .query_filtered::<(Entity, &AudioSink), With<BackgroundAudioMarker>>()
            .get_single(world)
            .ok()
    }
    fn loaded(world: &mut World) {
        let aa = world.resource::<AudioAssets>();
        let mut bg_inds = (0..aa.audio_bg.len()).collect_vec();
        bg_inds.shuffle(&mut thread_rng());
        let ar = AudioResource { bg_inds, bg_cur: 0 };
        let bg = aa.audio_bg[ar.bg_inds[0]].clone();
        world.insert_resource(ar);
        world.spawn((
            AudioBundle {
                source: bg,
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Remove,
                    volume: Volume::new(client_settings().volume_music),
                    ..default()
                },
            },
            BackgroundAudioMarker,
        ));
    }
    fn update(world: &mut World) {
        for q in FX_QUEUE.lock().unwrap().drain(..) {
            match q {
                SoundEffect::Click => {
                    Self::play(world.resource::<AudioAssets>().audio_click.clone(), world)
                }
                SoundEffect::Coin => {
                    Self::play(world.resource::<AudioAssets>().audio_coin.clone(), world)
                }
                SoundEffect::StartGame => Self::play(
                    world.resource::<AudioAssets>().audio_start_game.clone(),
                    world,
                ),
            }
        }
        let Some((entity, sink)) = Self::music_sink(world) else {
            return;
        };
        if sink.empty() {
            let mut ar = world.resource_mut::<AudioResource>();
            ar.bg_cur = (ar.bg_cur + 1) % ar.bg_inds.len();
            let bg_ind = ar.bg_inds[ar.bg_cur];
            let bg = world.resource::<AudioAssets>().audio_bg[bg_ind].clone();
            world
                .entity_mut(entity)
                .remove::<AudioSink>()
                .insert(AudioBundle {
                    source: bg,
                    settings: PlaybackSettings::REMOVE,
                });
        }
    }
}
