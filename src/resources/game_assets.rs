use spacetimedb_lib::de::serde::DeserializeWrapper;

use super::*;

static UNIT_REP: OnceCell<Representation> = OnceCell::new();
static STATUS_REP: OnceCell<Representation> = OnceCell::new();
static HERO_REP: OnceCell<Representation> = OnceCell::new();
static ANIMATIONS: OnceCell<HashMap<String, Anim>> = OnceCell::new();

static ALL: OnceCell<All> = OnceCell::new();
static GLOBAL_SETTINGS: OnceCell<GlobalSettings> = OnceCell::new();

pub fn all() -> &'static All {
    ALL.get().unwrap()
}
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
pub fn global_settings_local() -> &'static GlobalSettings {
    GLOBAL_SETTINGS.get().unwrap()
}

pub fn parse_content_tree() {
    const ASSETS: Dir = include_dir!("./assets/ron/");
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

    let all = All::from_dir("all".into(), ASSETS.get_dir("all").unwrap());
    ALL.set(all.unwrap()).unwrap();

    UNIT_REP
        .set(Representation::from_dir("unit_rep".to_owned(), &ASSETS).unwrap())
        .unwrap();
    STATUS_REP
        .set(Representation::from_dir("status_rep".to_owned(), &ASSETS).unwrap())
        .unwrap();
    HERO_REP
        .set(Representation::from_dir("hero_rep".to_owned(), &ASSETS).unwrap())
        .unwrap();
    let mut animations = HashMap::default();
    for f in ASSETS.get_dir("animation").unwrap().files() {
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
