use super::*;
use spacetimedb_lib::de::serde::DeserializeWrapper;
use spacetimedb_sdk::table::TableType;

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
    pub ability_defaults: HashMap<String, HashMap<VarName, VarValue>>,
    pub statuses: HashMap<String, PackedStatus>,
    pub vfxs: HashMap<String, Vfx>,
    pub colors: HashMap<String, Color32>,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct GlobalSettingsAsset {
    settings: DeserializeWrapper<GlobalSettings>,
}

impl GameAssets {
    pub fn get(world: &World) -> &Self {
        world.resource::<Self>()
    }
    pub fn cache_tables(world: &mut World) {
        let mut assets = world.resource_mut::<Self>();

        assets.global_settings = GlobalSettings::iter().exactly_one().ok().unwrap();

        assets.heroes.clear();
        for unit in BaseUnit::iter() {
            assets.heroes.insert(unit.name.clone(), unit.into());
        }
    }
    pub fn color(name: &str, world: &World) -> Color32 {
        Self::get(world).colors.get(name).unwrap().clone()
    }
    pub fn ability_default(name: &str, var: VarName, world: &World) -> VarValue {
        Self::get(world)
            .ability_defaults
            .get(name)
            .and_then(|m| m.get(&var))
            .cloned()
            .unwrap_or_default()
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

        let mut colors: HashMap<String, Color32> = default();
        let mut ability_defaults: HashMap<String, HashMap<VarName, VarValue>> = default();
        let mut abilities: HashMap<String, Ability> = default();
        let mut statuses: HashMap<String, PackedStatus> = default();
        let houses = world.resource::<Assets<House>>();
        let houses = HashMap::from_iter(handles.houses.iter().map(|(_, h)| {
            let house = houses.get(h).unwrap().clone();
            colors.insert(house.name.clone(), house.color.clone().into());
            abilities.extend(house.abilities.iter().map(|a| (a.name.clone(), a.clone())));
            ability_defaults.extend(house.defaults.iter().map(|(k, v)| (k.clone(), v.clone())));
            statuses.extend(house.statuses.iter().map(|s| (s.name.clone(), s.clone())));
            (house.name.clone(), house)
        }));
        let heroes = world.resource::<Assets<PackedUnit>>();
        let heroes = HashMap::from_iter(handles.heroes.iter().map(|(_, h)| {
            let hero = heroes.get(h).unwrap().clone();
            (hero.name.clone(), hero)
        }));
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
            colors,
            ability_defaults,
        };
        world.insert_resource(assets);
    }
}

fn name_from_path(path: &str) -> String {
    let (path, _) = path.split_once('.').unwrap();
    let from = path.rfind('/').unwrap_or_default();
    path.split_at(from).1.trim_matches('/').to_owned()
}
