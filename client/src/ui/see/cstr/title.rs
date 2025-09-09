use super::*;
use crate::ui::core::{CYAN, RED};
use bevy::color::Color;
use bevy::math::Vec2;
use convert_case::Casing;
use itertools::Itertools;
use schema::*;
use std::ops::Deref;
use utils_client::ToC32;

pub trait SFnTitleCstr {
    fn cstr_title(&self) -> Cstr;
}

// Implementations for basic types

impl<T: SFnTitleCstr> SFnTitleCstr for Vec<T> {
    fn cstr_title(&self) -> Cstr {
        self.into_iter().map(|i| i.cstr_title()).join(" ")
    }
}

impl<T: SFnTitleCstr> SFnTitleCstr for Box<T> {
    fn cstr_title(&self) -> Cstr {
        self.deref().cstr_title()
    }
}

impl SFnTitleCstr for String {
    fn cstr_title(&self) -> Cstr {
        self.clone()
    }
}

impl SFnTitleCstr for str {
    fn cstr_title(&self) -> Cstr {
        self.to_owned()
    }
}

impl SFnTitleCstr for &str {
    fn cstr_title(&self) -> Cstr {
        (*self).to_owned()
    }
}

impl SFnTitleCstr for u8 {
    fn cstr_title(&self) -> Cstr {
        self.to_string()
    }
}

impl SFnTitleCstr for u32 {
    fn cstr_title(&self) -> Cstr {
        self.to_string()
    }
}

impl SFnTitleCstr for u64 {
    fn cstr_title(&self) -> Cstr {
        self.to_string()
    }
}

impl SFnTitleCstr for f32 {
    fn cstr_title(&self) -> Cstr {
        format!("{self:.2}")
    }
}

impl SFnTitleCstr for f64 {
    fn cstr_title(&self) -> Cstr {
        format!("{self:.2}")
    }
}

impl SFnTitleCstr for i32 {
    fn cstr_title(&self) -> Cstr {
        self.to_string()
    }
}

impl SFnTitleCstr for bool {
    fn cstr_title(&self) -> Cstr {
        self.to_string()
    }
}

impl SFnTitleCstr for Vec2 {
    fn cstr_title(&self) -> Cstr {
        format!("{}×{}", self.x as i32, self.y as i32)
    }
}

impl SFnTitleCstr for Color {
    fn cstr_title(&self) -> Cstr {
        self.c32().cstr_title()
    }
}

impl SFnTitleCstr for Color32 {
    fn cstr_title(&self) -> Cstr {
        self.to_hex().color(*self)
    }
}

impl SFnTitleCstr for HexColor {
    fn cstr_title(&self) -> Cstr {
        let s = &self.0;
        format!("[{s} {s}]")
    }
}

impl SFnTitleCstr for VarName {
    fn cstr_title(&self) -> Cstr {
        self.as_ref().to_owned().color(self.color())
    }
}

impl SFnTitleCstr for VarValue {
    fn cstr_title(&self) -> Cstr {
        self.to_string()
    }
}

impl SFnTitleCstr for Expression {
    fn cstr_title(&self) -> Cstr {
        let base = match self {
            Self::r#if(..) => "if",
            Self::r#mod(..) => "mod",
            _ => self.as_ref(),
        };
        base.to_owned().color(self.color())
    }
}

impl SFnTitleCstr for PainterAction {
    fn cstr_title(&self) -> Cstr {
        self.as_ref().to_owned().color(CYAN)
    }
}

impl SFnTitleCstr for Trigger {
    fn cstr_title(&self) -> Cstr {
        let mut s = self.as_ref().to_owned().color(self.color());
        match self {
            Trigger::ChangeStat(var_name) => {
                s = format!("{} {}", s, SFnTitleCstr::cstr_title(var_name));
            }
            Trigger::BattleStart
            | Trigger::TurnEnd
            | Trigger::BeforeDeath
            | Trigger::ChangeOutgoingDamage
            | Trigger::ChangeIncomingDamage => {}
        }
        s
    }
}

impl SFnTitleCstr for MagicType {
    fn cstr_title(&self) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl SFnTitleCstr for Action {
    fn cstr_title(&self) -> Cstr {
        let inner_x = <Self as Injector<Expression>>::get_inner(self);
        let inner_a = <Self as Injector<Action>>::get_inner(self);
        let s = self.as_ref().to_owned().color(self.color());
        if !inner_x.is_empty() || !inner_a.is_empty() {
            let inner = inner_x
                .into_iter()
                .map(|x| x.cstr_expanded())
                .chain(inner_a.into_iter().map(|a| SFnTitleCstr::cstr_title(a)))
                .join(", ");
            format!("{s}[tl (]{inner}[tl )]")
        } else {
            s
        }
    }
}

impl SFnTitleCstr for Event {
    fn cstr_title(&self) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl SFnTitleCstr for ExpressionError {
    fn cstr_title(&self) -> Cstr {
        format!("{self}").color_style(RED, CstrStyle::Small)
    }
}

impl SFnTitleCstr for Reaction {
    fn cstr_title(&self) -> Cstr {
        let mut s = String::new();
        s.push_str(&self.trigger.cstr_title());
        s.push('\n');
        s.push_str(
            &self
                .actions
                .iter()
                .map(|a| format!("  {}", SFnTitleCstr::cstr_title(a)))
                .join("\n"),
        );
        s
    }
}

impl SFnTitleCstr for Material {
    fn cstr_title(&self) -> Cstr {
        format!("Material([th {}])", self.0.len())
    }
}

impl SFnTitleCstr for raw_nodes::NodeKind {
    fn cstr_title(&self) -> Cstr {
        if matches!(self, raw_nodes::NodeKind::None) {
            return "All".to_owned();
        }
        self.as_ref()
            .split_at(1)
            .1
            .to_case(convert_case::Case::Title)
    }
}

impl SFnTitleCstr for SoundEffect {
    fn cstr_title(&self) -> Cstr {
        self.as_ref().cstr_title()
    }
}

impl SFnTitleCstr for NUnit {
    fn cstr_title(&self) -> Cstr {
        self.unit_name.clone()
    }
}

impl SFnTitleCstr for NHouse {
    fn cstr_title(&self) -> Cstr {
        self.house_name.clone()
    }
}

impl SFnTitleCstr for NAbilityMagic {
    fn cstr_title(&self) -> Cstr {
        self.ability_name.clone()
    }
}

impl SFnTitleCstr for NStatusMagic {
    fn cstr_title(&self) -> Cstr {
        self.status_name.clone()
    }
}

impl SFnTitleCstr for NFusion {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NHouseColor {
    fn cstr_title(&self) -> Cstr {
        self.color.cstr_title()
    }
}

impl SFnTitleCstr for NAbilityDescription {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NAbilityEffect {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NStatusDescription {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NStatusBehavior {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NStatusRepresentation {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NUnitStats {
    fn cstr_title(&self) -> Cstr {
        format!("[red {}]/[yellow {}]", self.hp, self.pwr)
    }
}

impl SFnTitleCstr for NUnitState {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NUnitDescription {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NUnitRepresentation {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for NUnitBehavior {
    fn cstr_title(&self) -> Cstr {
        format!("[tw {}]", self.kind())
    }
}

impl SFnTitleCstr for AnimAction {
    fn cstr_title(&self) -> Cstr {
        self.as_ref().to_owned().color(PURPLE)
    }
}

impl SFnTitleCstr for Anim {
    fn cstr_title(&self) -> Cstr {
        self.actions.iter().map(|a| a.cstr_title()).join(" ")
    }
}

impl SFnTitleCstr for BattleAction {
    fn cstr_title(&self) -> Cstr {
        match self {
            BattleAction::strike(a, b) => format!("{a}|{b}"),
            BattleAction::damage(a, b, x) => format!("{a}>{b}-{x}"),
            BattleAction::heal(a, b, x) => format!("{a}>{b}+{x}"),
            BattleAction::death(a) => format!("x{a}"),
            BattleAction::var_set(a, var, value) => format!("{a}>${var}>{value}"),
            BattleAction::spawn(a) => format!("*{a}"),
            BattleAction::apply_status(a, status, charges, color) => {
                format!(
                    "+[{} {}]>{a}({charges})",
                    color.to_hex(),
                    status.status_name
                )
            }
            BattleAction::wait(t) => format!("~{t}"),
            BattleAction::vfx(_, vfx) => format!("vfx({vfx})"),
            BattleAction::send_event(e) => format!("event({e})"),
        }
    }
}
