use super::*;

/// Load and store shader programs
#[derive(Default)]
pub struct ShaderPrograms(HashMap<PathBuf, ugli::Program>);

impl ShaderPrograms {
    // full path
    pub fn get_program(&self, path: &PathBuf) -> &ugli::Program {
        &self
            .0
            .get(path)
            .expect(&format!("Shader not loaded {:?}", path))
    }

    pub fn insert_program(&mut self, file: PathBuf, program: ugli::Program) {
        self.0.insert(file, program);
    }

    fn program_loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::program_loader));
        let program = futures::executor::block_on(<ugli::Program as geng::LoadAsset>::load(
            &resources.geng.as_ref().unwrap(),
            path,
        ));
        match program {
            Ok(program) => {
                debug!("Load shader program {:?}", path);
                resources
                    .shader_programs
                    .insert_program(path.clone(), program)
            }
            Err(error) => error!("Failed to load shader program {:?} {}", path, error),
        }
    }

    pub fn shader_library_loader(
        resources: &mut Resources,
        path: &PathBuf,
        watcher: &mut FileWatcherSystem,
    ) {
        watcher.watch_file(path, Box::new(Self::shader_library_loader));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        paths.into_iter().for_each(|path| {
            Self::shader_library_program_loader(resources, &static_path().join(path), watcher);
        })
    }

    fn shader_library_program_loader(
        resources: &mut Resources,
        path: &PathBuf,
        watcher: &mut FileWatcherSystem,
    ) {
        watcher.watch_file(path, Box::new(Self::shader_library_program_loader));
        let geng = resources.geng.as_ref().unwrap();
        let program = &futures::executor::block_on(<String as geng::LoadAsset>::load(geng, &path));
        match program {
            Ok(program) => {
                debug!("Load shader library program {:?}", path);
                resources
                    .geng
                    .as_ref()
                    .unwrap()
                    .shader_lib()
                    .add(path.file_name().unwrap().to_str().unwrap(), program);
            }
            Err(error) => error!("Failed to load shader library program {:?} {}", path, error),
        }
    }
}

impl FileWatcherLoader for ShaderPrograms {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        paths.into_iter().for_each(|path| {
            Self::program_loader(resources, &static_path().join(path), watcher);
        })
    }
}
