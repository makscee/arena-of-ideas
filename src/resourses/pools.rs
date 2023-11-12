use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    heroes_handles: HashMap<String, Handle<PackedUnit>>,
    pub heroes: HashMap<String, PackedUnit>,
    #[asset(key = "pool.enemies", collection(typed, mapped))]
    enemies_handles: HashMap<String, Handle<PackedUnit>>,
    pub enemies: HashMap<String, PackedUnit>,
    #[asset(key = "pool.houses", collection(typed, mapped))]
    houses_handles: HashMap<String, Handle<House>>,
    pub houses: HashMap<String, House>,
    pub statuses: HashMap<String, PackedStatus>,
    pub abilities: HashMap<String, Ability>,
    #[asset(key = "pool.vfx", collection(typed, mapped))]
    vfx_handles: HashMap<String, Handle<Vfx>>,
    pub vfx: HashMap<String, Vfx>,
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
    pub fn get_vfx<'a>(name: &str, world: &'a World) -> Vfx {
        let name = &format!("ron/vfx/{name}.vfx.ron");
        Self::get(world).vfx.get(name).unwrap().clone()
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
        Self::setup_vfx(world);
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
        debug!("Setup {} houses", houses.len());
        world.get_resource_mut::<Pools>().unwrap().houses = houses;
    }

    pub fn setup_vfx(world: &mut World) {
        let vfx = HashMap::from_iter(
            world
                .get_resource::<Pools>()
                .unwrap()
                .vfx_handles
                .iter()
                .map(|(path, handle)| {
                    let vfx = world
                        .get_resource::<Assets<Vfx>>()
                        .unwrap()
                        .get(handle)
                        .unwrap()
                        .clone();

                    (path.to_owned(), vfx)
                }),
        );
        debug!("Setup {} vfx", vfx.len());
        world.get_resource_mut::<Pools>().unwrap().vfx = vfx;
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
        debug!("Setup {} statuses", statuses.len());
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
        debug!("Setup {} abilities", abilities.len());
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
        debug!("Setup {} heroes", heroes.len());
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
        debug!("Setup {} enemies", enemies.len());
        for (key, value) in enemies.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate enemy name: {key}")
            }
        }
    }
}

impl Plugin for PoolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), Self::setup);
    }
}