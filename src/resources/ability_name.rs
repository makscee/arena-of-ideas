use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityName {
    Shield,
    Defense,
    Dispel,
    Devour,
    Shoot,
    Rebirth,
    Grow,
    Reanimate,
    Poison,
    Swap,
    Immortality,
    LifeSteal,
    Empower,
    Mend,
    SummonTreant,
    Bounty,
    Discount,
    Bargain,
    Prosper,
}

impl fmt::Display for AbilityName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
