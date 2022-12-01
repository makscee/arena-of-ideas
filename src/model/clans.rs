use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum Clan {
    Warlocks,
    Protectors,
    Demons,
    Wizards,
    Common,
}

impl fmt::Display for Clan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Clan::Warlocks => write!(f, "Warlocks"),
            Clan::Protectors => write!(f, "Protectors"),
            Clan::Demons => write!(f, "Demons"),
            Clan::Wizards => write!(f, "Wizards"),
            Clan::Common => write!(f, "Common"),
        }
    }
}
