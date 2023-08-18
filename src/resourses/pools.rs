use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    pub heroes: HashMap<String, Handle<PackedUnit>>,
}
