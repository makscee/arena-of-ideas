use assets::{ANIMATIONS, HERO_REP, UNIT_REP};
use spacetimedb_lib::de::serde::DeserializeWrapper;

use super::*;

static HOUSES: OnceCell<HashMap<String, House>> = OnceCell::new();
static GLOBAL_SETTINGS: OnceCell<GlobalSettings> = OnceCell::new();

pub fn houses() -> &'static HashMap<String, House> {
    HOUSES.get().unwrap()
}
pub fn global_settings_local() -> &'static GlobalSettings {
    GLOBAL_SETTINGS.get().unwrap()
}

pub fn parse_content_tree() {
    let mut houses: HashMap<String, House> = default();
    const CONTENT_DIR: Dir = include_dir!("./assets/ron/");
    const GLOBAL_SETTINGS_STR: &str = include_str!("../.././assets/ron/_.global_settings.ron");

    GLOBAL_SETTINGS
        .set(
            ron::from_str::<DeserializeWrapper<GlobalSettings>>(GLOBAL_SETTINGS_STR)
                .unwrap()
                .0,
        )
        .unwrap();

    for dir in include_dir!("./assets/ron/")
        .get_dir("houses")
        .unwrap()
        .dirs()
    {
        let house = House::from_dir(dir.path().to_str().unwrap().to_string(), dir).unwrap();
        let name = house.get_var(VarName::name).unwrap().get_string().unwrap();
        houses.insert(name, house);
    }
    HOUSES.set(houses).unwrap();
    UNIT_REP
        .set(Representation::from_dir("unit_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    HERO_REP
        .set(Representation::from_dir("hero_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    let mut animations = HashMap::default();
    for f in CONTENT_DIR.get_dir("animations").unwrap().files() {
        let a: Vec<AnimAction> = ron::from_str(f.contents_utf8().unwrap()).unwrap();
        animations.insert(
            f.path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_end_matches(".ron")
                .to_owned(),
            Anim::new(a),
        );
    }
    dbg!(&animations);
    ANIMATIONS.set(animations).unwrap();
}
