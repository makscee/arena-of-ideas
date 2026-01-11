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
            | Action::use_ability(..)
            | Action::apply_status(..)
            | Action::set_status(..)
            | Action::change_status_stax(..)
            | Action::repeat(..) => 1,
        }
    }
}

impl Tier for Behavior {
    fn tier(&self) -> u8 {
        let trigger_tier = self.trigger.tier();
        let target_tier = self.target.tier();
        let effect_tier = self.effect.actions.iter().map(|a| a.tier()).sum::<u8>();
        (trigger_tier + target_tier + effect_tier) / 3
    }
}

impl<T> Tier for RhaiScript<T> {
    fn tier(&self) -> u8 {
        1
    }
}
