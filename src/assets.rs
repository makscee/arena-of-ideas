use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub units: UnitTemplates,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub player: Vec<UnitType>,
    pub spawn_points: HashMap<String, Vec2<Coord>>,
    pub waves: Vec<Wave>,
}
