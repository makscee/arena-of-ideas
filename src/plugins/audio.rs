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
    #[asset(key = "audio_strike")]
    audio_strike: Handle<AudioSource>,
    #[asset(key = "audio_pain")]
    audio_pain: Handle<AudioSource>,
    #[asset(key = "audio_death")]
    audio_death: Handle<AudioSource>,
    #[asset(key = "audio_status_add")]
    audio_status_add: Handle<AudioSource>,
    #[asset(key = "audio_status_remove")]
    audio_status_remove: Handle<AudioSource>,
    #[asset(key = "audio_victory")]
    audio_victory: Handle<AudioSource>,
    #[asset(key = "audio_defeat")]
    audio_defeat: Handle<AudioSource>,
    #[asset(key = "audio_inventory")]
    audio_inventory: Handle<AudioSource>,
}

#[derive(Resource, Debug)]
struct AudioResource {
    bg_inds: Vec<usize>,
    bg_cur: usize,
}

#[derive(Component)]
struct BackgroundAudioMarker;

#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, Default, EnumIter, AsRefStr, PartialEq, Eq,
)]
pub enum SoundEffect {
    #[default]
    Click,
    Coin,
    StartGame,
    Strike,
    Pain,
    Death,
    StatusAdd,
    StatusRemove,
    Victory,
    Defeat,
    Inventory,
}

static FX_QUEUE: Mutex<Vec<SoundEffect>> = Mutex::new(Vec::new());

impl AudioPlugin {
    pub fn set_music_volume(value: f32, world: &mut World) {
        if let Some((_, s)) = Self::background_sink(world) {
            s.set_volume(value);
        }
    }
    pub fn queue_sound(sfx: SoundEffect) {
        FX_QUEUE.lock().unwrap().push(sfx);
    }
    fn play(source: Handle<AudioSource>, world: &mut World) {
        world.spawn(AudioBundle {
            source,
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: Volume::new(client_settings().fx_volume()),
                ..default()
            },
        });
    }
    fn background_sink(world: &mut World) -> Option<(Entity, &AudioSink)> {
        world
            .query_filtered::<(Entity, &AudioSink), With<BackgroundAudioMarker>>()
            .get_single(world)
            .ok()
    }
    fn play_next_bg(world: &mut World) {
        let Some(mut ar) = world.get_resource_mut::<AudioResource>() else {
            return;
        };
        ar.bg_cur = (ar.bg_cur + 1) % ar.bg_inds.len();
        let bg_ind = ar.bg_inds[ar.bg_cur];
        let bg = world.resource::<AudioAssets>().audio_bg[bg_ind].clone();
        world.spawn((
            AudioBundle {
                source: bg,
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: Volume::new(client_settings().music_volume()),
                    ..default()
                },
            },
            BackgroundAudioMarker,
        ));
    }
    fn loaded(world: &mut World) {
        let aa = world.resource::<AudioAssets>();
        let mut bg_inds = (0..aa.audio_bg.len()).collect_vec();
        bg_inds.shuffle(&mut thread_rng());
        let ar = AudioResource { bg_inds, bg_cur: 0 };
        world.insert_resource(ar);
    }
    fn update(world: &mut World) {
        for q in FX_QUEUE.lock().unwrap().drain(..) {
            let aa = world.resource::<AudioAssets>();
            let sound = match q {
                SoundEffect::Click => aa.audio_click.clone(),
                SoundEffect::Coin => aa.audio_coin.clone(),
                SoundEffect::StartGame => aa.audio_start_game.clone(),
                SoundEffect::Strike => aa.audio_strike.clone(),
                SoundEffect::Pain => aa.audio_pain.clone(),
                SoundEffect::Death => aa.audio_death.clone(),
                SoundEffect::StatusAdd => aa.audio_status_add.clone(),
                SoundEffect::StatusRemove => aa.audio_status_remove.clone(),
                SoundEffect::Victory => aa.audio_victory.clone(),
                SoundEffect::Defeat => aa.audio_defeat.clone(),
                SoundEffect::Inventory => aa.audio_inventory.clone(),
            };
            Self::play(sound, world);
        }
        if Self::background_sink(world).is_none() {
            Self::play_next_bg(world);
        }
    }
}

impl ToCstr for SoundEffect {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr()
    }
}
