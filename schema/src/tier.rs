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
            | Action::set_target(..)
            | Action::add_target(..)
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability
            | Action::apply_status
            | Action::set_status(..)
            | Action::change_status_stax(..)
            | Action::repeat(..) => 1,
        }
    }
}

impl Tier for Reaction {
    fn tier(&self) -> u8 {
        let action_tiers = self.actions.iter().map(|a| a.tier()).sum::<u8>();
        (action_tiers + 1) / 2
    }
}
