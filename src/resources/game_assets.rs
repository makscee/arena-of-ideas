use assets::{HERO_REP, UNIT_REP};

use super::*;

static CONTENT_DIR: Dir = include_dir!("./assets/ron/modular/");
static HOUSES: OnceCell<HashMap<String, House>> = OnceCell::new();

pub fn houses() -> &'static HashMap<String, House> {
    HOUSES.get().unwrap()
}

pub fn parse_content_tree() {
    let mut houses: HashMap<String, House> = default();
    for dir in CONTENT_DIR.get_dir("houses").unwrap().dirs() {
        let house = House::from_dir(dir.path().to_str().unwrap().to_string(), dir).unwrap();
        let name = house.get_var(VarName::name).unwrap().get_string().unwrap();
        houses.insert(name, house);
    }
    UNIT_REP
        .set(Representation::from_dir("unit_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    HERO_REP
        .set(Representation::from_dir("hero_rep".to_owned(), &CONTENT_DIR).unwrap())
        .unwrap();
    HOUSES.set(houses).unwrap();
}
