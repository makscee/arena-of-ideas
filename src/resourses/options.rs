use super::*;

#[derive(AssetCollection, Resource)]
pub struct Options {
    #[asset(key = "unit.rep")]
    pub unit: Handle<Representation>,
    #[asset(key = "custom.battle")]
    pub custom_battle: Handle<BattleState>,
}

impl Options {
    pub fn get_unit_rep(world: &World) -> &Representation {
        world
            .get_resource::<Assets<Representation>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().unit)
            .unwrap()
    }
    pub fn get_custom_battle(world: &World) -> &BattleState {
        world
            .get_resource::<Assets<BattleState>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().custom_battle)
            .unwrap()
    }
}
