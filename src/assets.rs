use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub units: UnitTemplates,
    pub config: Config,
}

#[derive(geng::Assets, Deserialize, Clone)]
#[asset(json)]
pub struct Config {
    pub player: Vec<UnitType>,
    pub spawn_points: HashMap<String, Vec2<Coord>>,
    pub waves: Vec<Wave>,
}
