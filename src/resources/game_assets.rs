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

// impl All {
//     fn from_dir_new(parent: u64, path: String, dir: &Dir) -> Option<Self> {
//         let file = dir.files().next()?;
//         let id = u64::from_str(file.path().file_stem()?.to_str()?).unwrap();
//         let mut d = Self::default();
//         d.inject_data(file.contents_utf8()?);
//         d.id = Some(id);
//         d.parent = Some(parent);
//         d.players = dir
//             .get_dir(format!("{path}/{}", "players"))
//             .into_iter()
//             .flat_map(|d| d.dirs())
//             .filter_map(|d| Player::from_dir_new(id, d.path().to_string_lossy().to_string(), d))
//             .collect_vec();
//         Some(d)
//     }
// }

// impl Player {
//     fn from_dir_new(parent: u64, path: String, dir: &Dir) -> Option<Self> {
//         let file = dir.files().next()?;
//         let id = u64::from_str(file.path().file_stem()?.to_str()?).unwrap();
//         let mut d = Self::default();
//         let data = file.contents_utf8()?;
//         d.inject_data(data);
//         d.id = Some(id);
//         d.parent = Some(parent);
//         d.identity = PlayerIdentity::from_dir_new(id, format!("{path}/{}", "identity"), dir);
//         Some(d)
//     }
// }

// impl PlayerIdentity {
//     fn from_dir_new(parent: u64, path: String, dir: &Dir) -> Option<Self> {
//         let dir = dir.get_dir(path)?;
//         let file = dir.files().next()?;
//         let id = u64::from_str(file.path().file_stem()?.to_str()?).unwrap();
//         let mut d = Self::default();
//         let data = file.contents_utf8()?;
//         d.inject_data(data);
//         d.id = Some(id);
//         d.parent = Some(parent);
//         Some(d)
//     }
// }

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

    // let all = All::from_dir("all".into(), ASSETS.get_dir("all").unwrap());
    let dir = ASSETS.get_dir("all").unwrap();
    dbg!(dir);
    // dbg!(dir
    //     .get_dir("all/0/players/1741081733583167")
    //     .unwrap()
    //     .files()
    //     .next());
    // panic!();
    let all = All::from_dir_new(0, "all/0".into(), dir);
    dbg!(&all);
    panic!();
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
