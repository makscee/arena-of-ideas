use std::sync::mpsc::Receiver;

use notify::{DebouncedEvent, RecommendedWatcher, Watcher};

use super::*;

pub struct FileWatcherSystem {
    loaders: HashMap<PathBuf, Box<dyn Fn(&mut Resources, &PathBuf, &mut FileWatcherSystem)>>,
    to_reload: HashSet<PathBuf>,
    watcher: RecommendedWatcher,
    receiver: Receiver<DebouncedEvent>,
}

pub trait FileWatcherLoader {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem);
}

impl FileWatcherSystem {
    pub fn new() -> Self {
        let (tx, receiver) = std::sync::mpsc::channel();
        let watcher: RecommendedWatcher =
            notify::Watcher::new(tx, std::time::Duration::from_secs(1))
                .expect("Failed to initialize a watcher");
        Self {
            loaders: default(),
            to_reload: default(),
            watcher,
            receiver,
        }
    }

    pub fn watch_file(
        &mut self,
        path: &PathBuf,
        f: Box<dyn Fn(&mut Resources, &PathBuf, &mut FileWatcherSystem)>,
    ) {
        // f(resources, file, self);
        self.watch(&path);
        self.loaders.insert(path.clone(), f);
    }

    fn watch(&mut self, file: &PathBuf) {
        self.watcher
            .watch(file, notify::RecursiveMode::NonRecursive)
            .expect(&format!("Failed to start watching {:?}", &file));
    }

    fn handle_notify(&mut self, event: notify::DebouncedEvent) {
        debug!("Notify event: {event:?}");
        match event {
            DebouncedEvent::NoticeWrite(path) | DebouncedEvent::Create(path) => {
                self.mark_for_reload(&path)
            }
            DebouncedEvent::NoticeRemove(_) | DebouncedEvent::Remove(_) => {}
            DebouncedEvent::Error(error, path) => {
                error!("Notify error on path {path:?}: {error}");
            }
            _ => {}
        }
    }

    fn mark_for_reload(&mut self, file: &PathBuf) {
        self.to_reload.insert(file.clone());
    }
}

impl System for FileWatcherSystem {
    fn update(&mut self, _world: &mut legion::World, resources: &mut Resources) {
        resources.reload_triggered = !self.to_reload.is_empty();
        let mut loaders = HashMap::default();
        mem::swap(&mut loaders, &mut self.loaders);

        self.to_reload
            .drain()
            .collect_vec()
            .into_iter()
            .for_each(|path| (loaders.get(&path).unwrap())(resources, &path, self));
        self.loaders.extend(loaders.drain());

        use std::sync::mpsc::TryRecvError;
        match self.receiver.try_recv() {
            Ok(event) => self.handle_notify(event),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                error!("Disconnected from the channel");
            }
        }
    }
}
