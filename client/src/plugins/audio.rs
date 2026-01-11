use bevy::asset::AssetServer;
use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, Volume};
use rand::seq::SliceRandom;

use super::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_assets)
            .add_systems(OnEnter(GameState::Loaded), loaded)
            .add_systems(Update, update);
    }
}

fn load_assets(world: &mut World) {
    let asset_server = world.resource::<AssetServer>();

    let assets = AudioAssets {
        audio_bg: vec![
            asset_server.load("audio/bg/game theme 1.ogg"),
            asset_server.load("audio/bg/game theme 2.ogg"),
            asset_server.load("audio/bg/game theme 3.ogg"),
            asset_server.load("audio/bg/game theme 4.ogg"),
            asset_server.load("audio/bg/game theme 5.ogg"),
            asset_server.load("audio/bg/game theme 6.ogg"),
            asset_server.load("audio/bg/game theme 7.ogg"),
            asset_server.load("audio/bg/game theme 8.ogg"),
            asset_server.load("audio/bg/game theme 9.ogg"),
        ],
        audio_click: asset_server.load("audio/fx/click.ogg"),
        audio_coin: asset_server.load("audio/fx/coin.ogg"),
        audio_start_game: asset_server.load("audio/fx/field_open.ogg"),
        audio_strike: asset_server.load("audio/fx/insert_main.ogg"),
        audio_pain: asset_server.load("audio/fx/insert_wrong.ogg"),
        audio_death: asset_server.load("audio/fx/debuff.ogg"),
        audio_status_add: asset_server.load("audio/fx/insert_start.ogg"),
        audio_status_remove: asset_server.load("audio/fx/undo.ogg"),
        audio_victory: asset_server.load("audio/fx/field_complete.ogg"),
        audio_defeat: asset_server.load("audio/fx/lose_game.ogg"),
        audio_inventory: asset_server.load("audio/fx/absorb.ogg"),
    };

    world.insert_resource(assets);
    GameState::Loaded.set_next(world);
}

#[derive(Resource, Clone)]
pub struct AudioAssets {
    pub audio_bg: Vec<Handle<AudioSource>>,
    pub audio_click: Handle<AudioSource>,
    pub audio_coin: Handle<AudioSource>,
    pub audio_start_game: Handle<AudioSource>,
    pub audio_strike: Handle<AudioSource>,
    pub audio_pain: Handle<AudioSource>,
    pub audio_death: Handle<AudioSource>,
    pub audio_status_add: Handle<AudioSource>,
    pub audio_status_remove: Handle<AudioSource>,
    pub audio_victory: Handle<AudioSource>,
    pub audio_defeat: Handle<AudioSource>,
    pub audio_inventory: Handle<AudioSource>,
}

#[derive(Resource, Debug)]
struct AudioResource {
    bg_inds: Vec<usize>,
    bg_cur: usize,
}

#[derive(BevyComponent)]
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
        if let Some((_, mut s)) = background_sink(world) {
            s.set_volume(Volume::Linear(value));
        }
    }
    pub fn queue_sound(sfx: SoundEffect) {
        FX_QUEUE.lock().push(sfx);
    }
    fn play(source: Handle<AudioSource>, world: &mut World) {
        world.spawn((
            AudioPlayer::new(source),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: Volume::Linear(pd().client_settings.fx_volume()),
                ..default()
            },
        ));
    }
}

fn background_sink<'a>(world: &'a mut World) -> Option<(Entity, Mut<'a, AudioSink>)> {
    world
        .query_filtered::<(Entity, &mut AudioSink), With<BackgroundAudioMarker>>()
        .single_mut(world)
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
        AudioPlayer::new(bg),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::Linear(pd().client_settings.music_volume()),
            ..default()
        },
        BackgroundAudioMarker,
    ));
}

fn loaded(world: &mut World) {
    let aa = world.resource::<AudioAssets>();
    let mut bg_inds = (0..aa.audio_bg.len()).collect_vec();
    bg_inds.shuffle(&mut rng());
    let ar = AudioResource { bg_inds, bg_cur: 0 };
    world.insert_resource(ar);
}

fn update(world: &mut World) {
    for q in FX_QUEUE.lock().drain(..) {
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
        AudioPlugin::play(sound, world);
    }
    if background_sink(world).is_none() {
        play_next_bg(world);
    }
}

impl ToCstr for SoundEffect {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr()
    }
}
