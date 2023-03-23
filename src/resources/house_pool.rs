use super::*;

#[derive(Default)]
pub struct HousePool {
    houses: HashMap<HouseName, House>,
}

impl HousePool {
    pub fn try_get_ability_vars(&self, house: &HouseName, ability: &str) -> Option<&Vars> {
        self.houses
            .get(house)
            .and_then(|x| x.abilities.get(ability))
            .and_then(|x| Some(&x.default_vars))
    }

    pub fn try_get_ability_var_int(
        &self,
        house: &HouseName,
        ability: &str,
        var: &VarName,
    ) -> Option<i32> {
        self.try_get_ability_vars(house, ability)
            .and_then(|x| x.try_get_int(var))
    }

    pub fn get_color(&self, house: &HouseName) -> Rgba<f32> {
        self.houses.get(house).unwrap().color
    }

    pub fn get_ability(&self, house: &HouseName, ability: &str) -> &Ability {
        self.houses
            .get(house)
            .unwrap()
            .abilities
            .get(ability)
            .unwrap()
    }

    pub fn insert_house(&mut self, name: HouseName, house: House) {
        self.houses.insert(name, house);
    }
}

impl FileWatcherLoader for HousePool {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        paths.into_iter().for_each(|path| {
            House::loader(resources, &static_path().join(path), watcher);
        })
    }
}
