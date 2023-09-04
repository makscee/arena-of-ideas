use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    pub heroes: HashMap<String, Handle<PackedUnit>>,
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
}
