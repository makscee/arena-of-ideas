use super::*;

pub trait EnumColor {
    fn color(&self) -> Color32;
}

impl EnumColor for VarName {
    fn color(&self) -> Color32 {
        match self {
            VarName::hp => RED,
            VarName::pwr => YELLOW,
            VarName::lvl => PURPLE,
            VarName::xp => LIGHT_PURPLE,
            VarName::tier => GRAY,
            _ => low_contrast_text(),
        }
    }
}

impl EnumColor for Action {
    fn color(&self) -> Color32 {
        match self {
            Action::noop => low_contrast_text(),
            Action::deal_damage => RED,
            Action::heal_damage => GREEN,
            Action::use_ability => ORANGE,
            Action::apply_status => PURPLE,
            Action::set_status(..) => PURPLE,
            Action::change_status_stacks(..) => ORANGE,
            Action::debug(..) => high_contrast_text(),
            Action::set_value(..)
            | Action::add_value(..)
            | Action::subtract_value(..)
            | Action::add_target(..) => CYAN,
            Action::repeat(..) => PURPLE,
        }
    }
}

impl EnumColor for Trigger {
    fn color(&self) -> Color32 {
        match self {
            Trigger::BattleStart
            | Trigger::TurnEnd
            | Trigger::BeforeDeath
            | Trigger::ChangeOutgoingDamage
            | Trigger::ChangeIncomingDamage => YELLOW,
            Trigger::ChangeStat(var) => var.color(),
        }
    }
}

impl EnumColor for Expression {
    fn color(&self) -> Color32 {
        match self {
            Expression::one
            | Expression::zero
            | Expression::gt
            | Expression::owner
            | Expression::target
            | Expression::unit_size
            | Expression::pi
            | Expression::pi2
            | Expression::stacks
            | Expression::all_units
            | Expression::all_enemy_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::string(..)
            | Expression::f32(..)
            | Expression::f32_slider(..)
            | Expression::i32(..)
            | Expression::bool(..)
            | Expression::vec2(..)
            | Expression::value(..)
            | Expression::lua_i32(..)
            | Expression::lua_f32(..)
            | Expression::color(..) => high_contrast_text(),
            Expression::var(var)
            | Expression::target_var(var)
            | Expression::owner_var(var)
            | Expression::caster_var(var)
            | Expression::status_var(var)
            | Expression::var_or_zero(var) => var.color(),
            Expression::state_var(_x, _)
            | Expression::sin(_x)
            | Expression::cos(_x)
            | Expression::even(_x)
            | Expression::abs(_x)
            | Expression::floor(_x)
            | Expression::ceil(_x)
            | Expression::fract(_x)
            | Expression::sqr(_x)
            | Expression::unit_vec(_x)
            | Expression::rand(_x)
            | Expression::random_unit(_x)
            | Expression::to_f32(_x)
            | Expression::neg(_x) => YELLOW,
            Expression::vec2_ee(_a, _b)
            | Expression::str_macro(_a, _b)
            | Expression::sum(_a, _b)
            | Expression::sub(_a, _b)
            | Expression::mul(_a, _b)
            | Expression::div(_a, _b)
            | Expression::max(_a, _b)
            | Expression::min(_a, _b)
            | Expression::r#mod(_a, _b)
            | Expression::and(_a, _b)
            | Expression::or(_a, _b)
            | Expression::equals(_a, _b)
            | Expression::greater_then(_a, _b)
            | Expression::less_then(_a, _b)
            | Expression::fallback(_a, _b) => RED,
            Expression::r#if(_a, _b, _c) | Expression::oklch(_a, _b, _c) => PURPLE,
        }
    }
}

impl EnumColor for MagicType {
    fn color(&self) -> Color32 {
        match self {
            MagicType::Ability => ORANGE,
            MagicType::Status => PURPLE,
        }
    }
}
