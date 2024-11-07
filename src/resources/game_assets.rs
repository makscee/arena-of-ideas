use super::*;
use spacetimedb_lib::de::serde::DeserializeWrapper;

#[derive(AssetCollection, Resource)]
pub struct GameAssetsHandles {
    #[asset(key = "global_settings")]
    global_settings: Handle<GlobalSettingsAsset>,
    #[asset(key = "custom_battle")]
    custom_battle: Handle<BattleResource>,
    #[asset(key = "unit.rep")]
    unit_rep: Handle<Representation>,
    #[asset(key = "ghost.unit")]
    ghost: Handle<PackedUnit>,
    #[asset(key = "status.rep")]
    status_rep: Handle<Representation>,
    #[asset(key = "animations")]
    animations: Handle<Animations>,
    #[asset(key = "heroes", collection(typed, mapped))]
    heroes: HashMap<String, Handle<PackedUnit>>,
    #[asset(key = "houses", collection(typed, mapped))]
    houses: HashMap<String, Handle<House>>,
    #[asset(key = "vfxs", collection(typed, mapped))]
    vfxs: HashMap<String, Handle<Vfx>>,
}

#[derive(Debug, Clone)]
pub struct GameAssets {
    pub global_settings: GlobalSettings,
    pub custom_battle: BattleResource,
    pub unit_rep: Representation,
    pub status_rep: Representation,
    pub animations: Animations,
    pub ghost: PackedUnit,
    pub vfxs: HashMap<String, Vfx>,
    pub heroes: HashMap<String, PackedUnit>,
    pub houses: HashMap<String, House>,

    pub abilities: HashMap<String, Ability>,
    pub ability_defaults: HashMap<String, HashMap<VarName, VarValue>>,
    pub statuses: HashMap<String, PackedStatus>,
    pub summons: HashMap<String, PackedUnit>,
}

static GAME_ASSETS: OnceCell<RwLock<GameAssets>> = OnceCell::new();
pub fn game_assets() -> std::sync::RwLockReadGuard<'static, GameAssets> {
    GAME_ASSETS.get().unwrap().read().unwrap()
}

lazy_static! {
    static ref NAME_COLORS: Mutex<HashMap<String, Color32>> = Mutex::new(HashMap::new());
    static ref NAME_DEFINITIONS: Mutex<HashMap<String, Cstr>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize, Asset, TypePath)]
pub struct GlobalSettingsAsset {
    settings: DeserializeWrapper<GlobalSettings>,
}

pub fn try_name_color(name: &str) -> Option<Color32> {
    NAME_COLORS.lock().unwrap().get(name).cloned()
}
pub fn name_color(name: &str) -> Color32 {
    try_name_color(name).unwrap_or(VISIBLE_LIGHT)
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
    pub fn cache_tables() {
        info!("Cache tables start");
        let global_settings = cn()
            .db
            .global_settings()
            .always_zero()
            .find(&0)
            .expect("Assets not synced");
        let mut heroes: HashMap<String, PackedUnit> = default();
        for unit in cn().db.base_unit().iter() {
            match unit.pool {
                UnitPool::Game => {
                    heroes.insert(unit.name.clone(), unit.into());
                }
                UnitPool::Summon => {}
            }
        }
        let mut houses: HashMap<String, House> = default();
        for house in cn().db.house().iter() {
            houses.insert(house.name.clone(), house.into());
        }
        let ga = game_assets().clone();
        let unit_rep = ga.unit_rep;
        let ghost = ga.ghost;
        let status_rep = ga.status_rep;
        let animations = ga.animations;
        let vfxs = ga.vfxs;
        let assets = LoadingPlugin::pack_game_assets(
            global_settings,
            default(),
            unit_rep,
            ghost,
            status_rep,
            animations,
            heroes,
            houses,
            vfxs,
        );
        *GAME_ASSETS.get().unwrap().write().unwrap() = assets;
    }
    pub fn ability_default(name: &str, var: VarName) -> VarValue {
        game_assets()
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
    pub strike: Anim,
}

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), Self::setup);
    }
}

impl LoadingPlugin {
    fn pack_game_assets(
        global_settings: GlobalSettings,
        custom_battle: BattleResource,
        unit_rep: Representation,
        ghost: PackedUnit,
        status_rep: Representation,
        animations: Animations,
        heroes: HashMap<String, PackedUnit>,
        houses: HashMap<String, House>,
        vfxs: HashMap<String, Vfx>,
    ) -> GameAssets {
        let mut colors = HashMap::default();
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
            }
            ability_defaults.extend(house.defaults.iter().map(|(k, v)| (k.clone(), v.clone())));
            for ability in house.abilities.iter() {
                abilities.insert(ability.name.clone(), ability.clone());
                colors.insert(ability.name.clone(), color);
            }
            for unit in house.summons.iter() {
                colors.insert(unit.name.clone(), color);
                summons.insert(unit.name.clone(), unit.clone());
            }
        }
        for (name, hero) in &heroes {
            colors.insert(name.clone(), *colors.get(&hero.houses[0]).unwrap());
        }
        *NAME_COLORS.lock().unwrap() = colors;
        for status in statuses.values() {
            definitions.insert(status.name.clone(), Cstr::parse(&status.description));
        }
        for ability in abilities.values() {
            definitions.insert(ability.name.clone(), Cstr::parse(&ability.description));
        }
        GameAssets {
            global_settings,
            custom_battle,
            unit_rep,
            ghost,
            status_rep,
            animations,
            heroes,
            houses,
            abilities,
            statuses,
            vfxs,
            ability_defaults,
            summons,
        }
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
            .resource::<Assets<BattleResource>>()
            .get(&handles.custom_battle)
            .unwrap()
            .clone();
        let unit_rep = world
            .resource::<Assets<Representation>>()
            .get(&handles.unit_rep)
            .unwrap()
            .clone();
        let ghost = world
            .resource::<Assets<PackedUnit>>()
            .get(&handles.ghost)
            .unwrap()
            .clone();
        let status_rep = world
            .resource::<Assets<Representation>>()
            .get(&handles.status_rep)
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
        let ga = Self::pack_game_assets(
            global_settings,
            custom_battle,
            unit_rep,
            ghost,
            status_rep,
            animations,
            heroes,
            houses,
            vfxs,
        );
        GAME_ASSETS.set(RwLock::new(ga)).unwrap();
    }
}

fn name_from_path(path: &str) -> String {
    let (path, _) = path.split_once('.').unwrap();
    let from = path.rfind('/').unwrap_or_default();
    path.split_at(from).1.trim_matches('/').to_owned()
}
