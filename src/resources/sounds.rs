use super::*;

#[derive(Default)]
pub struct Sounds(HashMap<PathBuf, geng::Sound>);

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SoundType {
    Click,
}

impl Sounds {
    pub fn insert_sound(file: PathBuf, sound: geng::Sound, resources: &mut Resources) {
        resources.sounds.0.insert(file, sound);
    }

    pub fn get_sound<'a>(file: &PathBuf, resources: &'a Resources) -> &'a geng::Sound {
        resources.sounds.0.get(file).unwrap()
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
        Self::get_sound(
            &static_path().join(resources.options.sounds.get(&r#type).unwrap()),
            resources,
        )
        .play();
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
