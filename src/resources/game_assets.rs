use std::fs::{create_dir_all, write};

use spacetimedb_lib::de::serde::DeserializeWrapper;
use spacetimedb_sats::serde::SerdeWrapper;

use super::*;

static UNIT_REP: OnceCell<Representation> = OnceCell::new();
static STATUS_REP: OnceCell<Representation> = OnceCell::new();
static HERO_REP: OnceCell<Representation> = OnceCell::new();
static ANIMATIONS: OnceCell<HashMap<String, Anim>> = OnceCell::new();

static ALL: OnceCell<All> = OnceCell::new();
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

    // let all = All::from_dir("all".into(), ASSETS.get_dir("all").unwrap());
    // let dir = ASSETS.get_dir("all").unwrap();
    // dbg!(dir);
    // dbg!(dir
    //     .get_dir("all/0/players/1741081733583167")
    //     .unwrap()
    //     .files()
    //     .next());
    // panic!();
    // let all = All::from_dir_new(0, "all/0".into(), dir);
    // dbg!(&all);
    // panic!();
    // ALL.set(all.unwrap()).unwrap();

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

pub struct GameAssets;

#[derive(Serialize, Deserialize, Default)]
struct IncubatorData {
    nodes: Vec<SerdeWrapper<TIncubator>>,
    links: Vec<SerdeWrapper<TIncubatorLinks>>,
    votes: Vec<SerdeWrapper<TIncubatorVotes>>,
}

impl GameAssets {
    pub fn update_files() {
        let nodes = cn()
            .db
            .incubator_nodes()
            .iter()
            .sorted_by_key(|n| n.id)
            .map(|n| SerdeWrapper::new(n))
            .collect_vec();
        let links = cn()
            .db
            .incubator_links()
            .iter()
            .sorted_by_cached_key(|l| l.key.clone())
            .map(|l| SerdeWrapper::new(l))
            .collect_vec();
        let votes = cn()
            .db
            .incubator_votes()
            .iter()
            .sorted_by_cached_key(|v| v.key.clone())
            .map(|v| SerdeWrapper::new(v))
            .collect_vec();
        let data = IncubatorData {
            nodes,
            links,
            votes,
        };
        data.save();
    }
}

impl IncubatorData {
    fn save(&self) {
        let path = "./assets/ron/links/";
        create_dir_all(path).unwrap();
        let nodes = to_ron_string(&self.nodes);
        debug!("nodes:\n{nodes}");
        let links = to_ron_string(&self.links);
        debug!("links:\n{links}");
        let votes = to_ron_string(&self.votes);
        debug!("votes:\n{votes}");
        write(path.to_owned() + "nodes.ron", nodes).unwrap();
        write(path.to_owned() + "links.ron", links).unwrap();
        write(path.to_owned() + "votes.ron", votes).unwrap();
    }
}
fn to_ron_string<T: Serialize>(value: &T) -> String {
    to_string_pretty(value, PrettyConfig::new().depth_limit(1)).unwrap()
}
