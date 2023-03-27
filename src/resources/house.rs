use super::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct House {
    pub name: HouseName,
    pub color: Rgba<f32>,
    pub abilities: HashMap<AbilityName, Ability>,
    #[serde(default)]
    pub statuses: HashMap<AbilityName, Status>,
}

#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone, Copy, enum_iterator::Sequence,
)]
pub enum HouseName {
    Vampires,
    Dragons,
    Robots,
    Snakes,
    Thieves,
    Exorcists,
    Warriors,
    Demons,
    Necromancers,
    Archers,
    Clerics,
    Titans,
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
            resources
                .status_pool
                .define_status(name.to_string(), status)
        });
        house.abilities.iter().for_each(|(name, ability)| {
            AbilityPool::define_ability(resources, name, ability, house.color, house.name);
        });
        resources.house_pool.insert_house(house.name, house);
    }
}
