use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityName {
    Fortitude,
    Mend,
    Enrage,
    Chaotic,

    Grow,
    LifeSteal,
    Shoot,
    Devour,
    Empower,
    Shield,
    SummonTreant,
    Defense,
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
