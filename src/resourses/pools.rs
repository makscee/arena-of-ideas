use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    pub heroes: HashMap<String, Handle<PackedUnit>>,
    #[asset(key = "pool.houses", collection(typed, mapped))]
    pub houses: HashMap<String, Handle<House>>,
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

    pub fn get_status<'a>(status: &str, house: &str, world: &'a World) -> &'a PackedStatus {
        Self::get_house(house, world)
            .statuses
            .iter()
            .find(|s| s.name.eq(status))
            .unwrap()
    }
}
