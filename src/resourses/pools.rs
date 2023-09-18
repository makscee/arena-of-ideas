use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    pub heroes: HashMap<String, Handle<PackedUnit>>,
    #[asset(key = "pool.houses", collection(typed, mapped))]
    pub houses: HashMap<String, Handle<House>>,
    pub statuses: HashMap<String, PackedStatus>,
}

impl Pools {
    pub fn heroes(world: &World) -> HashMap<String, &PackedUnit> {
        let result = world
            .get_resource::<Pools>()
            .unwrap()
            .heroes
            .iter()
            .map(|(name, handle)| {
                (
                    name.to_owned(),
                    world
                        .get_resource::<Assets<PackedUnit>>()
                        .unwrap()
                        .get(handle)
                        .unwrap(),
                )
            });
        HashMap::from_iter(result)
    }

    pub fn get_house<'a>(name: &str, world: &'a World) -> &'a House {
        world
            .get_resource::<Assets<House>>()
            .unwrap()
            .get(
                world
                    .get_resource::<Pools>()
                    .unwrap()
                    .houses
                    .get(&Self::full_file_name(name))
                    .unwrap(),
            )
            .unwrap()
    }

    fn full_file_name(name: &str) -> String {
        format!("ron/houses/{}.house.ron", name.to_lowercase())
    }

    pub fn get_ability<'a>(ability: &str, house: &str, world: &'a World) -> &'a Ability {
        Self::get_house(house, world)
            .abilities
            .iter()
            .find(|a| a.name.eq(ability))
            .unwrap()
    }

    pub fn get_status<'a>(status: &str, world: &'a World) -> &'a PackedStatus {
        world
            .get_resource::<Pools>()
            .unwrap()
            .statuses
            .get(status)
            .unwrap()
    }
}

pub struct PoolsPlugin;

impl PoolsPlugin {
    pub fn setup(world: &mut World) {
        debug!("status setup");
        let statuses = world
            .get_resource::<Pools>()
            .unwrap()
            .houses
            .values()
            .map(|handle| {
                world
                    .get_resource::<Assets<House>>()
                    .unwrap()
                    .get(handle)
                    .unwrap()
                    .statuses
                    .clone()
            })
            .flatten()
            .collect_vec();
        let pool = &mut world.get_resource_mut::<Pools>().unwrap().statuses;
        for (key, value) in statuses.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate status name: {key}")
            }
        }
    }
}

impl Plugin for PoolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), Self::setup);
    }
}
