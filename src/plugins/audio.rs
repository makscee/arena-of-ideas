use bevy_kira_audio::AudioApp;

use super::*;

pub struct AudioPlugin;

#[derive(Resource)]
struct BackgroundChannel {
    handle: Handle<AudioInstance>,
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<BackgroundChannel>()
            .insert_resource(BackgroundChannel { handle: default() })
            .add_systems(OnEnter(GameState::Shop), Self::start_background)
            .add_systems(OnExit(GameState::Shop), Self::stop_background);
    }
}

impl AudioPlugin {
    fn start_background(world: &mut World) {
        let track = world
            .resource::<AssetServer>()
            .load("ron/audio/shop_bg.ogg.ron");
        let channel = world.resource::<AudioChannel<BackgroundChannel>>();
        let handle = channel.play(track).looped().handle();
        world.insert_resource(BackgroundChannel { handle });
    }

    fn stop_background(world: &mut World) {
        let channel = world.resource::<AudioChannel<BackgroundChannel>>();
        channel.stop();
    }

    pub fn background_position(world: &mut World) -> Option<f64> {
        let instance = world.resource::<BackgroundChannel>().handle.clone();
        let channel = world.resource::<AudioChannel<BackgroundChannel>>();
        channel.state(&instance).position()
    }
}
