use std::sync::mpsc::Receiver;

use notify::{DebouncedEvent, RecommendedWatcher, Watcher};

use super::*;

pub struct FileWatcherSystem {
    files: HashMap<PathBuf, Box<dyn FnMut(&mut Resources, &PathBuf)>>,
    to_reload: HashSet<PathBuf>,
    watcher: RecommendedWatcher,
    receiver: Receiver<DebouncedEvent>,
}

impl FileWatcherSystem {
    pub fn new() -> Self {
        let (tx, receiver) = std::sync::mpsc::channel();
        let watcher: RecommendedWatcher =
            notify::Watcher::new(tx, std::time::Duration::from_secs(1))
                .expect("Failed to initialize a watcher");
        Self {
            files: default(),
            to_reload: default(),
            watcher,
            receiver,
        }
    }

    pub fn load_and_watch_file(
        &mut self,
        resources: &mut Resources,
        file: &PathBuf,
        mut f: Box<dyn FnMut(&mut Resources, &PathBuf)>,
    ) {
        f(resources, file);
        self.watch(&file);
        self.files.insert(file.clone(), f);
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
        self.to_reload.drain().for_each(|file| {
            self.files
                .get_mut(&file)
                .expect(&format!("Can't find file to reload: {:?}", file))(
                resources, &file
            );
        });
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
