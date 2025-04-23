use std::fs::{create_dir_all, read, read_to_string, write};

use include_dir::{DirEntry, File};
use spacetimedb_lib::de::serde::DeserializeWrapper;
use spacetimedb_sats::serde::SerdeWrapper;

use super::*;

static UNIT_REP: OnceCell<NRepresentation> = OnceCell::new();
static STATUS_REP: OnceCell<NRepresentation> = OnceCell::new();
static ANIMATIONS: OnceCell<HashMap<String, Anim>> = OnceCell::new();

static GLOBAL_SETTINGS: OnceCell<GlobalSettings> = OnceCell::new();

pub fn unit_rep() -> &'static NRepresentation {
    UNIT_REP.get().unwrap()
}
pub fn status_rep() -> &'static NRepresentation {
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
        .set(NRepresentation::from_dir("unit_rep".to_owned(), assets()).unwrap())
        .unwrap();
    STATUS_REP
        .set(NRepresentation::from_dir("status_rep".to_owned(), assets()).unwrap())
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

// #[derive(Serialize, Deserialize, Default)]
// pub struct IncubatorData {
//     pub nodes: Vec<SerdeWrapper<TIncubator>>,
//     pub links: Vec<SerdeWrapper<TIncubatorLinks>>,
//     pub votes: Vec<SerdeWrapper<TIncubatorVotes>>,
// }

// impl GameAssets {
//     pub fn update_files() {
//         let nodes = cn()
//             .db
//             .incubator_nodes()
//             .iter()
//             .sorted_by_key(|n| n.id)
//             .map(|n| SerdeWrapper::new(n))
//             .collect_vec();
//         let links = cn()
//             .db
//             .incubator_links()
//             .iter()
//             .sorted_by_cached_key(|l| l.key.clone())
//             .map(|l| SerdeWrapper::new(l))
//             .collect_vec();
//         let votes = cn()
//             .db
//             .incubator_votes()
//             .iter()
//             .sorted_by_cached_key(|v| v.key.clone())
//             .map(|v| SerdeWrapper::new(v))
//             .collect_vec();
//         let data = IncubatorData {
//             nodes,
//             links,
//             votes,
//         };
//         data.save();
//     }
// }

// impl IncubatorData {
//     fn save(&self) {
//         let path = "./assets/ron/links/";
//         create_dir_all(path).unwrap();
//         let nodes = to_ron_string(&self.nodes);
//         debug!("nodes:\n{nodes}");
//         let links = to_ron_string(&self.links);
//         debug!("links:\n{links}");
//         let votes = to_ron_string(&self.votes);
//         debug!("votes:\n{votes}");
//         write(path.to_owned() + "nodes.ron", nodes).unwrap();
//         write(path.to_owned() + "links.ron", links).unwrap();
//         write(path.to_owned() + "votes.ron", votes).unwrap();
//     }
//     pub fn load() -> Self {
//         let path = "./assets/ron/links/";
//         let nodes = read_to_string(path.to_owned() + "nodes.ron").unwrap();
//         let links = read_to_string(path.to_owned() + "links.ron").unwrap();
//         let votes = read_to_string(path.to_owned() + "votes.ron").unwrap();
//         Self {
//             nodes: ron::from_str(&nodes).unwrap(),
//             links: ron::from_str(&links).unwrap(),
//             votes: ron::from_str(&votes).unwrap(),
//         }
//     }
// }
fn to_ron_string<T: Serialize>(value: &T) -> String {
    to_string_pretty(value, PrettyConfig::new().depth_limit(1)).unwrap()
}
