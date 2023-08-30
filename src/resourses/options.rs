use super::*;

#[derive(AssetCollection, Resource)]
pub struct Options {
    #[asset(key = "unit.rep")]
    pub unit: Handle<Representation>,
    #[asset(key = "custom.battle")]
    pub custom_battle: Handle<BattleState>,
    #[asset(key = "anim")]
    pub animations: Handle<Animations>,
}

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath)]
#[uuid = "e96699ce-cabf-461f-86df-913957687d72"]
pub struct Animations(HashMap<AnimationType, Animation>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnimationType {
    BeforeStrike,
    Strike,
    AfterStrike,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Animation {
    pub var: VarName,
    pub change: Vec<Change>,
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
