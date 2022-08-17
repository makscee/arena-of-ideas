use super::*;

#[derive(Deserialize, Clone, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
pub struct SimulationConfig {
    pub simulations: Vec<SimulationType>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SimulationType {
    Balance {
        unit: RegexUnit,
        repeats: usize,
    },
    Units {
        squad: Vec<RegexUnit>,
        enemies: Vec<RegexUnit>,
        repeats: usize,
        clan_bonuses: Vec<usize>,
    },
    Rounds {
        squad: Vec<RegexUnit>,
        clan_bonuses: Vec<usize>,
        from: usize,
        to: usize,
        repeats: usize,
    },
}

pub type RegexUnit = String;
