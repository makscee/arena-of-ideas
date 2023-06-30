use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityName {
    Vitality,
    Mend,
    Strength,
    Defense,
    Weakness,
    Decay,

    Grow,
    LifeSteal,
    Shoot,
    Devour,
    Empower,
    Shield,
    SummonTreant,
    Dispel,
    Reanimate,
    Swap,
    Poison,
    Rebirth,
}

impl fmt::Display for AbilityName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
