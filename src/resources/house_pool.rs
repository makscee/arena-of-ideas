use super::*;

#[derive(Default)]
pub struct HousePool {
    houses: HashMap<HouseName, House>,
}

impl HousePool {
    pub fn get_color(&self, house: &HouseName) -> Rgba<f32> {
        self.houses.get(house).unwrap().color
    }

    pub fn insert_house(&mut self, name: HouseName, house: House) {
        self.houses.insert(name, house);
    }
}

impl FileWatcherLoader for HousePool {
    fn loader(resources: &mut Resources, _: &PathBuf, watcher: &mut FileWatcherSystem) {
        enum_iterator::all::<HouseName>()
            .map(|x| {
                let name = format!("houses/{:?}.json", x).to_lowercase();
                static_path().join(name)
            })
            .for_each(|path| {
                House::loader(resources, &static_path().join(path), watcher);
            });
    }
}
