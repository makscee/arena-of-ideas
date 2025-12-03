use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, AsRefStr, EnumString, Display)]
pub enum CreationPart {
    HouseName,
    HouseColor,
    AbilityName,
    StatusName,
    AbilityDescription,
    StatusDescription,
    AbilityImplementation,
    StatusImplementation,

    UnitName,
    UnitDescription,
    UnitImplementation,
    UnitStats,
    UnitRepresentation,
}

impl CreationPart {
    pub fn depend_on(self) -> Option<Self> {
        match self {
            CreationPart::HouseName => None,
            CreationPart::HouseColor | CreationPart::AbilityName | CreationPart::StatusName => {
                Some(CreationPart::HouseName)
            }
            CreationPart::AbilityDescription => Some(CreationPart::AbilityName),
            CreationPart::StatusDescription => Some(CreationPart::StatusName),
            CreationPart::AbilityImplementation => Some(CreationPart::AbilityDescription),
            CreationPart::StatusImplementation => Some(CreationPart::StatusDescription),

            CreationPart::UnitName => None,
            CreationPart::UnitDescription => Some(CreationPart::UnitName),
            CreationPart::UnitImplementation
            | CreationPart::UnitStats
            | CreationPart::UnitRepresentation => Some(CreationPart::UnitDescription),
        }
    }

    pub fn is_complete(parts: &Vec<Self>, is_unit: bool) -> bool {
        if is_unit {
            for p in [
                CreationPart::UnitName,
                CreationPart::UnitDescription,
                CreationPart::UnitStats,
                CreationPart::UnitImplementation,
            ] {
                if !parts.contains(&p) {
                    return false;
                }
            }
            true
        } else {
            parts.contains(&Self::HouseName)
                && parts.contains(&Self::HouseColor)
                && (parts.contains(&Self::AbilityName)
                    && parts.contains(&Self::AbilityDescription)
                    && parts.contains(&Self::AbilityImplementation)
                    || parts.contains(&Self::StatusName)
                        && parts.contains(&Self::StatusDescription)
                        && parts.contains(&Self::StatusImplementation))
        }
    }

    pub fn is_unit(self) -> bool {
        match self {
            CreationPart::UnitName
            | CreationPart::UnitDescription
            | CreationPart::UnitImplementation
            | CreationPart::UnitStats
            | CreationPart::UnitRepresentation => true,
            _ => false,
        }
    }
}
