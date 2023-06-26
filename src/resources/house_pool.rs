use super::*;

#[derive(Default)]
pub struct HousePool {
    houses: HashMap<HouseName, House>,
}

impl HousePool {
    pub fn get_color(house: &HouseName, resources: &Resources) -> Rgba<f32> {
        resources.house_pool.houses.get(house).unwrap().color
    }

    pub fn insert_house(name: HouseName, house: House, resources: &mut Resources) {
        resources.house_pool.houses.insert(name, house);
    }
}

impl FileWatcherLoader for HousePool {
    fn load(resources: &mut Resources, _: &PathBuf, watcher: &mut FileWatcherSystem) {
        enum_iterator::all::<HouseName>()
            .map(|x| {
                let name = format!("houses/{:?}.json", x).to_lowercase();
                static_path().join(name)
            })
            .for_each(|path| {
                House::load(resources, &path, watcher);
            });
    }
}
