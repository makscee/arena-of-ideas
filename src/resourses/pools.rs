use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    heroes_handles: HashMap<String, Handle<PackedUnit>>,
    #[asset(key = "pool.enemies", collection(typed, mapped))]
    enemies_handles: HashMap<String, Handle<PackedUnit>>,
    #[asset(key = "pool.houses", collection(typed, mapped))]
    houses_handles: HashMap<String, Handle<House>>,
    pub statuses: HashMap<String, PackedStatus>,
    pub houses: HashMap<String, House>,
    pub abilities: HashMap<String, Ability>,
    pub heroes: HashMap<String, PackedUnit>,
    pub enemies: HashMap<String, PackedUnit>,
}

impl Pools {
    pub fn get<'a>(world: &'a World) -> &'a Self {
        world.get_resource::<Pools>().unwrap()
    }
    pub fn get_mut(world: &mut World) -> Mut<Self> {
        world.get_resource_mut::<Pools>().unwrap()
    }

    pub fn get_status<'a>(name: &str, world: &'a World) -> &'a PackedStatus {
        Self::get(world).statuses.get(name).unwrap()
    }
    pub fn get_ability<'a>(name: &str, world: &'a World) -> &'a Ability {
        Self::get(world).abilities.get(name).unwrap()
    }
}

pub struct PoolsPlugin;

impl PoolsPlugin {
    pub fn setup(world: &mut World) {
        Self::setup_houses(world);
        Self::setup_statuses(world);
        Self::setup_abilities(world);
        Self::setup_heroes(world);
        Self::setup_enemies(world);
    }

    pub fn setup_houses(world: &mut World) {
        let houses = HashMap::from_iter(
            world
                .get_resource::<Pools>()
                .unwrap()
                .houses_handles
                .values()
                .map(|handle| {
                    let house = world
                        .get_resource::<Assets<House>>()
                        .unwrap()
                        .get(handle)
                        .unwrap()
                        .clone();
                    (house.name.to_owned(), house)
                }),
        );
        debug!("Setup houses: {houses:#?}");
        world.get_resource_mut::<Pools>().unwrap().houses = houses;
    }

    pub fn setup_statuses(world: &mut World) {
        let statuses = Pools::get(world)
            .houses
            .iter()
            .map(|(_, h)| {
                let mut statuses = h.statuses.clone();
                for status in statuses.iter_mut() {
                    status
                        .state
                        .init(VarName::Color, VarValue::Color(h.color.clone().into()));
                }
                statuses
            })
            .flatten()
            .collect_vec();
        let pool = &mut Pools::get_mut(world).statuses;
        debug!("Setup statuses: {statuses:#?}");
        for (key, value) in statuses.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate status name: {key}")
            }
        }
    }

    pub fn setup_abilities(world: &mut World) {
        let abilities = Pools::get(world)
            .houses
            .iter()
            .map(|(_, h)| h.abilities.clone())
            .flatten()
            .collect_vec();
        let pool = &mut Pools::get_mut(world).abilities;
        debug!("Setup abilities: {abilities:#?}");
        for (key, value) in abilities.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate ability name: {key}")
            }
        }
    }

    pub fn setup_heroes(world: &mut World) {
        let heroes = world
            .get_resource::<Pools>()
            .unwrap()
            .heroes_handles
            .values()
            .map(|handle| {
                world
                    .get_resource::<Assets<PackedUnit>>()
                    .unwrap()
                    .get(handle)
                    .unwrap()
                    .clone()
            })
            .collect_vec();
        let pool = &mut Pools::get_mut(world).heroes;
        debug!("Setup heroes: {heroes:#?}");
        for (key, value) in heroes.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate hero name: {key}")
            }
        }
    }

    pub fn setup_enemies(world: &mut World) {
        let enemies = world
            .get_resource::<Pools>()
            .unwrap()
            .enemies_handles
            .values()
            .map(|handle| {
                world
                    .get_resource::<Assets<PackedUnit>>()
                    .unwrap()
                    .get(handle)
                    .unwrap()
                    .clone()
            })
            .collect_vec();
        let pool = &mut Pools::get_mut(world).enemies;
        debug!("Setup enemies: {enemies:#?}");
        for (key, value) in enemies.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate enemy name: {key}")
            }
        }
    }
}

impl Plugin for PoolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), Self::setup);
    }
}
