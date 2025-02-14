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
            VarName::tier => YELLOW,
            _ => VISIBLE_DARK,
        }
    }
}

impl EnumColor for Action {
    fn color(&self) -> Color32 {
        match self {
            Action::Noop => VISIBLE_DARK,
            Action::DealDamage => RED,
            Action::HealDamage => GREEN,
            Action::UseAbility => ORANGE,
            Action::Debug(..) => VISIBLE_LIGHT,
            Action::SetValue(..)
            | Action::AddValue(..)
            | Action::SubtractValue(..)
            | Action::AddTarget(..) => CYAN,
            Action::Repeat(..) => PURPLE,
        }
    }
}

impl EnumColor for Trigger {
    fn color(&self) -> Color32 {
        match self {
            Trigger::BattleStart | Trigger::TurnEnd | Trigger::BeforeDeath => YELLOW,
            Trigger::ChangeStat(var) => var.color(),
        }
    }
}

impl EnumColor for Expression {
    fn color(&self) -> Color32 {
        match self {
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::Owner
            | Expression::Target
            | Expression::UnitSize
            | Expression::PI
            | Expression::PI2
            | Expression::AllUnits
            | Expression::AllEnemyUnits
            | Expression::AllAllyUnits
            | Expression::AllOtherAllyUnits
            | Expression::AdjacentAllyUnits
            | Expression::AdjacentBack
            | Expression::AdjacentFront
            | Expression::S(..)
            | Expression::F(..)
            | Expression::FSlider(..)
            | Expression::I(..)
            | Expression::B(..)
            | Expression::V2(..)
            | Expression::V(..)
            | Expression::C(..) => VISIBLE_LIGHT,
            Expression::Var(var) => var.color(),
            Expression::StateVar(_x, _)
            | Expression::Sin(_x)
            | Expression::Cos(_x)
            | Expression::Even(_x)
            | Expression::Abs(_x)
            | Expression::Floor(_x)
            | Expression::Ceil(_x)
            | Expression::Fract(_x)
            | Expression::Sqr(_x)
            | Expression::UnitVec(_x)
            | Expression::Rand(_x)
            | Expression::RandomUnit(_x)
            | Expression::ToF(_x) => YELLOW,
            Expression::V2EE(_a, _b)
            | Expression::Macro(_a, _b)
            | Expression::Sum(_a, _b)
            | Expression::Sub(_a, _b)
            | Expression::Mul(_a, _b)
            | Expression::Div(_a, _b)
            | Expression::Max(_a, _b)
            | Expression::Min(_a, _b)
            | Expression::Mod(_a, _b)
            | Expression::And(_a, _b)
            | Expression::Or(_a, _b)
            | Expression::Equals(_a, _b)
            | Expression::GreaterThen(_a, _b)
            | Expression::LessThen(_a, _b)
            | Expression::Fallback(_a, _b) => RED,
            Expression::If(_a, _b, _c) | Expression::Oklch(_a, _b, _c) => PURPLE,
        }
    }
}
