use super::*;

#[derive(AssetCollection, Resource)]
pub struct Options {
    #[asset(key = "unit.rep")]
    pub unit: Handle<Representation>,
    #[asset(key = "status.rep")]
    pub status: Handle<Representation>,
    #[asset(key = "slot.rep")]
    pub slot: Handle<Representation>,
    #[asset(key = "initial.tower")]
    pub initial_tower: Handle<Tower>,
    #[asset(key = "custom.battle")]
    pub custom_battle: Handle<CustomBattleData>,
    #[asset(key = "anim")]
    pub animations: Handle<Animations>,
}

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath)]
#[uuid = "e96699ce-cabf-461f-86df-913957687d72"]
pub struct Animations(HashMap<AnimationType, Anim>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum AnimationType {
    BeforeStrike,
    AfterStrike,
}

impl Animations {
    pub fn get(&self, t: AnimationType) -> &Anim {
        self.0.get(&t).unwrap()
    }
}

impl Options {
    pub fn get_unit_rep(world: &World) -> &Representation {
        world
            .get_resource::<Assets<Representation>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().unit)
            .unwrap()
    }
    pub fn get_status_rep(world: &World) -> &Representation {
        world
            .get_resource::<Assets<Representation>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().status)
            .unwrap()
    }
    pub fn get_slot_rep(world: &World) -> &Representation {
        world
            .get_resource::<Assets<Representation>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().slot)
            .unwrap()
    }
    pub fn get_custom_battle(world: &World) -> &CustomBattleData {
        world
            .get_resource::<Assets<CustomBattleData>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().custom_battle)
            .unwrap()
    }
    pub fn get_animations(world: &World) -> &Animations {
        world
            .get_resource::<Assets<Animations>>()
            .unwrap()
            .get(&world.get_resource::<Options>().unwrap().animations)
            .unwrap()
    }
}
