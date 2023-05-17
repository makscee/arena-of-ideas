use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityName {
    Fortitude,
    Mend,
    Enrage,
}

impl fmt::Display for AbilityName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
