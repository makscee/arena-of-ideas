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
    pub vfxs: HashMap<String, Vfx>,
    pub heroes: HashMap<String, PackedUnit>,
    pub houses: HashMap<String, House>,

    pub abilities: HashMap<String, Ability>,
    pub ability_defaults: HashMap<String, HashMap<VarName, VarValue>>,
    pub statuses: HashMap<String, PackedStatus>,
    pub summons: HashMap<String, PackedUnit>,
}

lazy_static! {
    static ref NAME_COLORS: Mutex<HashMap<String, Color32>> = Mutex::new(HashMap::new());
    static ref NAME_DEFINITIONS: Mutex<HashMap<String, Cstr>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize, Asset, TypePath)]
pub struct GlobalSettingsAsset {
    settings: DeserializeWrapper<GlobalSettings>,
}

pub fn name_color(name: &str) -> Color32 {
    NAME_COLORS
        .lock()
        .unwrap()
        .get(name)
        .with_context(|| format!("Failed to find color for {name}"))
        .unwrap()
        .clone()
}
pub fn definition(name: &str) -> Cstr {
    NAME_DEFINITIONS.lock().unwrap().get(name).unwrap().clone()
}
pub fn definition_names() -> Vec<String> {
    NAME_DEFINITIONS
        .lock()
        .unwrap()
        .keys()
        .cloned()
        .collect_vec()
}

impl GameAssets {
    pub fn get(world: &World) -> &Self {
        world.resource::<Self>()
    }
    pub fn cache_tables(world: &mut World) {
        let global_settings = GlobalSettings::iter().exactly_one().ok().unwrap();
        let mut heroes: HashMap<String, PackedUnit> = default();
        for unit in BaseUnit::iter() {
            heroes.insert(unit.name.clone(), unit.into());
        }
        let mut houses: HashMap<String, House> = default();
        for house in THouse::iter() {
            houses.insert(house.name.clone(), house.into());
        }
        let ga = world.remove_resource::<GameAssets>().unwrap();
        let unit_rep = ga.unit_rep;
        let animations = ga.animations;
        let vfxs = ga.vfxs;
        LoadingPlugin::save_assets(
            global_settings,
            default(),
            unit_rep,
            animations,
            heroes,
            houses,
            vfxs,
            world,
        );
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
    fn save_assets(
        global_settings: GlobalSettings,
        custom_battle: BattleData,
        unit_rep: Representation,
        animations: Animations,
        heroes: HashMap<String, PackedUnit>,
        houses: HashMap<String, House>,
        vfxs: HashMap<String, Vfx>,
        world: &mut World,
    ) {
        let mut colors = NAME_COLORS.lock().unwrap();
        let mut definitions = NAME_DEFINITIONS.lock().unwrap();
        let mut ability_defaults: HashMap<String, HashMap<VarName, VarValue>> = default();
        let mut abilities: HashMap<String, Ability> = default();
        let mut statuses: HashMap<String, PackedStatus> = default();
        let mut summons: HashMap<String, PackedUnit> = default();
        for (_, house) in houses.iter() {
            let color: Color32 = house.color.clone().into();
            colors.insert(house.name.clone(), color);
            for status in house.statuses.iter() {
                statuses.insert(status.name.clone(), status.clone());
                colors.insert(status.name.clone(), color);
                definitions.insert(status.name.clone(), Cstr::parse(&status.description));
            }
            ability_defaults.extend(house.defaults.iter().map(|(k, v)| (k.clone(), v.clone())));
            for ability in house.abilities.iter() {
                abilities.insert(ability.name.clone(), ability.clone());
                colors.insert(ability.name.clone(), color);
                definitions.insert(ability.name.clone(), Cstr::parse(&ability.description));
            }
            for unit in house.summons.iter() {
                let mut unit = unit.clone();
                unit.rarity = -1;
                summons.insert(unit.name.clone(), unit.clone());
            }
        }
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
            ability_defaults,
            summons,
        };
        world.insert_resource(assets);
    }
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
        let vfxs = world.resource::<Assets<Vfx>>();
        let vfxs = HashMap::from_iter(
            handles
                .vfxs
                .iter()
                .map(|(name, h)| (name_from_path(name), vfxs.get(h).unwrap().clone())),
        );
        let houses = world.resource::<Assets<House>>();
        let houses = HashMap::from_iter(handles.houses.iter().map(|(_, h)| {
            let house = houses.get(h).unwrap().clone();
            (house.name.clone(), house)
        }));
        let heroes = world.resource::<Assets<PackedUnit>>();
        let heroes = HashMap::from_iter(handles.heroes.iter().map(|(_, h)| {
            let hero = heroes.get(h).unwrap().clone();
            (hero.name.clone(), hero)
        }));
        Self::save_assets(
            global_settings,
            custom_battle,
            unit_rep,
            animations,
            heroes,
            houses,
            vfxs,
            world,
        );
    }
}

fn name_from_path(path: &str) -> String {
    let (path, _) = path.split_once('.').unwrap();
    let from = path.rfind('/').unwrap_or_default();
    path.split_at(from).1.trim_matches('/').to_owned()
}
