use spacetimedb_lib::de::serde::DeserializeWrapper;

use super::*;

static UNIT_REP: OnceCell<Representation> = OnceCell::new();
static STATUS_REP: OnceCell<Representation> = OnceCell::new();
static HERO_REP: OnceCell<Representation> = OnceCell::new();
static ANIMATIONS: OnceCell<HashMap<String, Anim>> = OnceCell::new();

static HOUSES: OnceCell<HashMap<String, House>> = OnceCell::new();
static GLOBAL_SETTINGS: OnceCell<GlobalSettings> = OnceCell::new();

pub fn unit_rep() -> &'static Representation {
    UNIT_REP.get().unwrap()
}
pub fn status_rep() -> &'static Representation {
    STATUS_REP.get().unwrap()
}
pub fn hero_rep() -> &'static Representation {
    HERO_REP.get().unwrap()
}
pub fn animations() -> &'static HashMap<String, Anim> {
    ANIMATIONS.get().unwrap()
}
pub fn houses() -> &'static HashMap<String, House> {
    HOUSES.get().unwrap()
}
pub fn global_settings_local() -> &'static GlobalSettings {
    GLOBAL_SETTINGS.get().unwrap()
}

pub fn parse_content_tree() {
    let mut houses: HashMap<String, House> = default();
    const CONTENT_DIR: Dir = include_dir!("./assets/ron/");
    const GLOBAL_SETTINGS_STR: &str = include_str!("../../assets/ron/_.global_settings.ron");
    const DESCRIPTIONS: &str = include_str!("../../assets/ron/descriptions.ron");

    GLOBAL_SETTINGS
        .set(
            ron::from_str::<DeserializeWrapper<GlobalSettings>>(GLOBAL_SETTINGS_STR)
                .unwrap()
                .0,
        )
        .unwrap();

    let descriptions: Descriptions = ron::from_str(DESCRIPTIONS).unwrap();
    descriptions.set();

    for dir in include_dir!("./assets/ron/")
        .get_dir("houses")
        .unwrap()
        .dirs()
    {
        let house = House::from_dir(dir.path().to_str().unwrap().to_string(), dir).unwrap();
        let strings = house.to_strings_root();
        dbg!(&strings);
        let d = House::from_strings(0, &strings);
        dbg!(d);
        let name = house.get_var(VarName::name).unwrap().get_string().unwrap();
        houses.insert(name, house);
        // dir.extract(PathBuf::from_str("./assets/ron/extract_test/").unwrap())
        //     .unwrap();
        dbg!(dir);
    }
    HOUSES.set(houses).unwrap();
    UNIT_REP
        .set(Representation::from_dir("unit_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    STATUS_REP
        .set(Representation::from_dir("status_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    HERO_REP
        .set(Representation::from_dir("hero_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    let mut animations = HashMap::default();
    for f in CONTENT_DIR.get_dir("animation").unwrap().files() {
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
    ANIMATIONS.set(animations).unwrap();
}
