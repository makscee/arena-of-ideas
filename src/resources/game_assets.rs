use spacetimedb_lib::de::serde::DeserializeWrapper;

use super::*;

static UNIT_REP: OnceCell<NUnitRepresentation> = OnceCell::new();
static STATUS_REP: OnceCell<NStatusRepresentation> = OnceCell::new();
static ANIMATIONS: OnceCell<HashMap<String, Anim>> = OnceCell::new();

static GLOBAL_SETTINGS: OnceCell<GlobalSettings> = OnceCell::new();

pub fn unit_rep() -> &'static NUnitRepresentation {
    UNIT_REP.get().unwrap()
}
pub fn status_rep() -> &'static NStatusRepresentation {
    STATUS_REP.get().unwrap()
}
pub fn animations() -> &'static HashMap<String, Anim> {
    ANIMATIONS.get().unwrap()
}
pub fn global_settings_local() -> &'static GlobalSettings {
    GLOBAL_SETTINGS.get().unwrap()
}
pub fn assets() -> &'static Dir<'static> {
    const ASSETS: Dir = include_dir!("./assets/ron/");
    &ASSETS
}

pub fn parse_content_tree() {
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
    // let house = NHouse {
    //     id: Some(10),
    //     parent: None,
    //     entity: None,
    //     house_name: "Wizards".into(),
    //     color: Some(NHouseColor {
    //         id: Some(11),
    //         parent: Some(10),
    //         entity: None,
    //         color: "#ff000ff".to_owned().into(),
    //     }),
    //     action_ability: None,
    //     status_ability: None,
    //     units: Vec::new(),
    // };
    // let core = NCore {
    //     id: Some(1),
    //     parent: None,
    //     entity: None,
    //     houses: [house].to_vec(),
    // };
    // let core = NCore::from_dir(0, "core".into(), &ASSETS).unwrap();
    // dbg!(&core);
    // let dir = core.to_dir("core".into());
    // let dir = Dir::new("core", dir);
    // dbg!(&dir);
    // let path = "./assets/ron/export_test/";
    // std::fs::create_dir_all(format!("{path}{}", dir.path().to_str().unwrap())).unwrap();
    // dir.extract(path).unwrap();
    UNIT_REP
        .set(NUnitRepresentation::from_dir("unit_rep".to_owned(), assets()).unwrap())
        .unwrap();
    STATUS_REP
        .set(NStatusRepresentation::from_dir("status_rep".to_owned(), assets()).unwrap())
        .unwrap();
    let mut animations = HashMap::default();
    for f in assets().get_dir("animation").unwrap().files() {
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

pub struct GameAssets;
