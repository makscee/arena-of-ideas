use super::*;

pub struct Shop {
    pub pool: HashMap<PathBuf, usize>,
    pub world: legion::World,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            pool: default(),
            world: default(),
        }
    }
}

impl Shop {
    pub fn load(&mut self, templates: &HashMap<PathBuf, UnitTemplate>) {
        self.pool = HashMap::from_iter(templates.keys().map(|path| (path.clone(), 3)));
    }
}
