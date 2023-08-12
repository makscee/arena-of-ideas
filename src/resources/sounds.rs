use strum_macros::Display;

use super::*;

#[derive(Default)]
pub struct Sounds(HashMap<PathBuf, geng::Sound>);

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Hash, Copy, Clone, Display)]
pub enum SoundType {
    Click,
    Buff,
    Debuff,
    Coin,
    HitMelee,
    HitRange,
    LevelUp,
    Victory,
    Defeat,
    StartGame,
    Spawn,
    Music,
}

impl Sounds {
    pub fn insert_sound(file: PathBuf, sound: geng::Sound, resources: &mut Resources) {
        resources.sounds.0.insert(file, sound);
    }

    pub fn get_sound<'a>(file: &PathBuf, resources: &'a Resources) -> &'a geng::Sound {
        resources.sounds.0.get(file).unwrap()
    }

    pub fn get_sound_mut<'a>(file: &PathBuf, resources: &'a mut Resources) -> &'a mut geng::Sound {
        resources.sounds.0.get_mut(file).unwrap()
    }

    fn sound_loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::sound_loader));
        let sound = futures::executor::block_on(<geng::Sound as geng::LoadAsset>::load(
            &resources.geng.as_ref().unwrap(),
            path,
        ));
        match sound {
            Ok(sound) => {
                debug!("Load sound file {:?}", path);
                Sounds::insert_sound(path.clone(), sound, resources)
            }
            Err(error) => error!("Failed to load sound file {:?} {}", path, error),
        }
    }

    pub fn play_sound(r#type: SoundType, resources: &Resources) {
        debug!("Play sound {}", r#type);
        Self::get_sound(
            &static_path().join(resources.options.sounds.get(&r#type).unwrap()),
            resources,
        )
        .play();
    }

    pub fn play_loop(r#type: SoundType, resources: &mut Resources) {
        let mut sound = Self::get_sound_mut(
            &static_path().join(resources.options.sounds.get(&r#type).unwrap()),
            resources,
        );
        sound.looped = true;
        let mut sound = sound.effect();
        sound.play();
        resources.sound_data.loops.insert(r#type, sound);
    }

    pub fn stop_loop(r#type: SoundType, resources: &mut Resources) {
        if let Some((_, mut sound)) = resources.sound_data.loops.remove_entry(&r#type) {
            sound.stop()
        }
    }
}

impl FileWatcherLoader for Sounds {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        paths.into_iter().for_each(|path| {
            Self::sound_loader(resources, &static_path().join(path), watcher);
        })
    }
}
