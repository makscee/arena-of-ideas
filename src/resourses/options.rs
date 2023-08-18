use super::*;

#[derive(AssetCollection, Resource)]
pub struct Options {
    #[asset(key = "rep.unit")]
    pub unit: Handle<Representation>,
}
