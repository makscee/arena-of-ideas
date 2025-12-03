use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, AsRefStr, EnumString, Display)]
pub enum CreationPhase {
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

impl CreationPhase {
    pub fn depend_on(self) -> Option<Self> {
        match self {
            CreationPhase::HouseName => None,
            CreationPhase::HouseColor | CreationPhase::AbilityName | CreationPhase::StatusName => {
                Some(CreationPhase::HouseName)
            }
            CreationPhase::AbilityDescription => Some(CreationPhase::AbilityName),
            CreationPhase::StatusDescription => Some(CreationPhase::StatusName),
            CreationPhase::AbilityImplementation => Some(CreationPhase::AbilityDescription),
            CreationPhase::StatusImplementation => Some(CreationPhase::StatusDescription),

            CreationPhase::UnitName => None,
            CreationPhase::UnitDescription => Some(CreationPhase::UnitName),
            CreationPhase::UnitImplementation
            | CreationPhase::UnitStats
            | CreationPhase::UnitRepresentation => Some(CreationPhase::UnitDescription),
        }
    }

    pub fn is_complete(phases: &Vec<Self>, is_unit: bool) -> bool {
        if is_unit {
            for p in [
                CreationPhase::UnitName,
                CreationPhase::UnitDescription,
                CreationPhase::UnitStats,
                CreationPhase::UnitImplementation,
            ] {
                if !phases.contains(&p) {
                    return false;
                }
            }
            true
        } else {
            phases.contains(&Self::HouseName)
                && phases.contains(&Self::HouseColor)
                && (phases.contains(&Self::AbilityName)
                    && phases.contains(&Self::AbilityDescription)
                    && phases.contains(&Self::AbilityImplementation)
                    || phases.contains(&Self::StatusName)
                        && phases.contains(&Self::StatusDescription)
                        && phases.contains(&Self::StatusImplementation))
        }
    }

    pub fn is_unit(self) -> bool {
        match self {
            CreationPhase::UnitName
            | CreationPhase::UnitDescription
            | CreationPhase::UnitImplementation
            | CreationPhase::UnitStats
            | CreationPhase::UnitRepresentation => true,
            _ => false,
        }
    }
}
