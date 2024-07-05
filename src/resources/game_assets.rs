use super::*;
use spacetimedb_lib::de::serde::DeserializeWrapper;

#[derive(AssetCollection, Resource)]
pub struct GameAssetsHandles {
    #[asset(key = "global_settings")]
    global_settings: Handle<GlobalSettingsAsset>,
    #[asset(key = "custom_battle")]
    custom_battle: Handle<BattleData>,
    #[asset(key = "unit.rep")]
    unit_rep: Handle<Representation>,
    #[asset(key = "animations")]
    animations: Handle<Animations>,
    #[asset(key = "heroes", collection(typed, mapped))]
    heroes: HashMap<String, Handle<PackedUnit>>,
    #[asset(key = "houses", collection(typed, mapped))]
    houses: HashMap<String, Handle<House>>,
    #[asset(key = "vfxs", collection(typed, mapped))]
    vfxs: HashMap<String, Handle<Vfx>>,
}

#[derive(Resource, Debug, Clone)]
pub struct GameAssets {
    pub global_settings: GlobalSettings,
    pub custom_battle: BattleData,
    pub unit_rep: Representation,
    pub animations: Animations,

    pub heroes: HashMap<String, PackedUnit>,
    pub houses: HashMap<String, House>,
    pub abilities: HashMap<String, Ability>,
    pub statuses: HashMap<String, PackedStatus>,
    pub vfxs: HashMap<String, Vfx>,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct GlobalSettingsAsset {
    settings: DeserializeWrapper<GlobalSettings>,
}

impl GameAssets {
    pub fn get(world: &World) -> &Self {
        world.resource::<Self>()
    }
}

#[derive(Deserialize, Asset, TypePath, Debug, Clone)]
pub struct Animations {
    pub before_strike: Anim,
    pub move_to_slot: Anim,
}

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), Self::setup);
    }
}

impl LoadingPlugin {
    fn setup(world: &mut World) {
        let handles = world.resource::<GameAssetsHandles>();
        let global_settings = world
            .resource::<Assets<GlobalSettingsAsset>>()
            .get(&handles.global_settings)
            .unwrap()
            .settings
            .0
            .clone();
        let custom_battle = world
            .resource::<Assets<BattleData>>()
            .get(&handles.custom_battle)
            .unwrap()
            .clone();
        let unit_rep = world
            .resource::<Assets<Representation>>()
            .get(&handles.unit_rep)
            .unwrap()
            .clone();
        let animations = world
            .resource::<Assets<Animations>>()
            .get(&handles.animations)
            .unwrap()
            .clone();

        let heroes = world.resource::<Assets<PackedUnit>>();
        let heroes = HashMap::from_iter(handles.heroes.iter().map(|(_, h)| {
            let hero = heroes.get(h).unwrap().clone();
            (hero.name.clone(), hero)
        }));
        let houses = world.resource::<Assets<House>>();
        let houses = HashMap::from_iter(handles.houses.iter().map(|(_, h)| {
            let house = houses.get(h).unwrap().clone();
            (house.name.clone(), house)
        }));
        let abilities = HashMap::from_iter(
            houses
                .values()
                .flat_map(|h| &h.abilities)
                .cloned()
                .map(|a| (a.name.clone(), a)),
        );
        let statuses = HashMap::from_iter(
            houses
                .values()
                .flat_map(|h| &h.statuses)
                .cloned()
                .map(|a| (a.name.clone(), a)),
        );
        let vfxs = world.resource::<Assets<Vfx>>();
        let vfxs = HashMap::from_iter(
            handles
                .vfxs
                .iter()
                .map(|(name, h)| (name_from_path(name), vfxs.get(h).unwrap().clone())),
        );

        let assets = GameAssets {
            global_settings,
            custom_battle,
            unit_rep,
            animations,
            heroes,
            houses,
            abilities,
            statuses,
            vfxs,
        };
        world.insert_resource(assets);
    }
}

fn name_from_path(path: &str) -> String {
    let (path, _) = path.split_once('.').unwrap();
    let from = path.rfind('/').unwrap_or_default();
    path.split_at(from).1.trim_matches('/').to_owned()
}
