use super::*;

#[derive(AssetCollection, Resource)]
pub struct Options {
    #[asset(key = "unit.rep")]
    pub unit: Handle<Representation>,
    #[asset(key = "status.rep")]
    pub status: Handle<Representation>,
    #[asset(key = "slot.rep")]
    pub slot: Handle<Representation>,
    #[asset(key = "custom.battle")]
    pub custom_battle: Handle<BattleData>,
    #[asset(key = "anim")]
    pub animations: Handle<Animations>,
    #[cfg_attr(debug_assertions, asset(key = "dev_options"))]
    #[cfg_attr(not(debug_assertions), asset(key = "prod_options"))]
    pub options: Handle<OptionsData>,
}

#[derive(Asset, Serialize, Deserialize, Debug, TypePath)]
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

#[derive(Asset, Serialize, Deserialize, Debug, TypePath)]
pub struct OptionsData {
    pub address: String,
    pub server: String,
}

impl Options {
    pub fn get_unit_rep(world: &World) -> &Representation {
        world
            .resource::<Assets<Representation>>()
            .get(&world.resource::<Options>().unit)
            .unwrap()
    }
    pub fn get_status_rep(world: &World) -> &Representation {
        world
            .resource::<Assets<Representation>>()
            .get(&world.resource::<Options>().status)
            .unwrap()
    }
    pub fn get_slot_rep(world: &World) -> &Representation {
        world
            .resource::<Assets<Representation>>()
            .get(&world.resource::<Options>().slot)
            .unwrap()
    }
    pub fn get_custom_battle(world: &World) -> &BattleData {
        world
            .resource::<Assets<BattleData>>()
            .get(&world.resource::<Options>().custom_battle)
            .unwrap()
    }
    pub fn get_animations(world: &World) -> &Animations {
        world
            .resource::<Assets<Animations>>()
            .get(&world.resource::<Options>().animations)
            .unwrap()
    }
    pub fn get_data(world: &World) -> &OptionsData {
        world
            .resource::<Assets<OptionsData>>()
            .get(&world.resource::<Options>().options)
            .unwrap()
    }
}
