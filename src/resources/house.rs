use super::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct House {
    pub name: HouseName,
    pub color: Rgba<f32>,
    pub abilities: HashMap<String, Ability>,
    #[serde(default)]
    pub statuses: HashMap<String, Status>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum HouseName {
    Vampires,
    Dragons,
    Robots,
    Snakes,
    Thieves,
    Exorcists,
}

impl FileWatcherLoader for House {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let house: House = futures::executor::block_on(load_json(path)).unwrap();
        house.statuses.iter().for_each(|(name, status)| {
            let mut status = status.clone();
            if status.color.is_none() {
                status.color = Some(house.color);
            }
            resources.status_pool.define_status(name.clone(), status)
        });
        house.abilities.iter().for_each(|(name, ability)| {
            resources
                .definitions
                .insert(name.clone(), house.color, ability.description.clone())
        });
        resources.house_pool.insert_house(house.name, house);
    }
}
