use super::*;

pub struct Shop {
    pub pool: HashMap<PathBuf, usize>,
    pub money: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            pool: default(),
            money: default(),
        }
    }
}

impl Shop {
    pub fn load(&mut self, pool: &UnitTemplatesPool) {
        self.pool = HashMap::from_iter(pool.heroes.keys().map(|path| (path.clone(), 3)));
    }
}
