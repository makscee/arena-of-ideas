use super::*;

pub trait Tier {
    fn tier(&self) -> u8;
}

impl Tier for Action {
    fn tier(&self) -> u8 {
        match self {
            Action::noop | Action::debug(..) => 0,
            Action::set_value(..)
            | Action::add_value(..)
            | Action::subtract_value(..)
            | Action::add_target(..)
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability
            | Action::repeat(..) => 1,
        }
    }
}
