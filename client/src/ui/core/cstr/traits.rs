use super::*;

// Implementations for basic types

impl<T: ToCstr> ToCstr for Vec<T> {
    fn cstr(&self) -> Cstr {
        self.into_iter().map(|i| i.cstr()).join(" ")
    }
}

impl<T: ToCstr> ToCstr for Box<T> {
    fn cstr(&self) -> Cstr {
        self.deref().cstr()
    }
}

impl ToCstr for String {
    fn cstr(&self) -> Cstr {
        self.clone()
    }
}

impl ToCstr for str {
    fn cstr(&self) -> Cstr {
        self.to_owned()
    }
}

impl ToCstr for &str {
    fn cstr(&self) -> Cstr {
        (*self).to_owned()
    }
}

impl ToCstr for u8 {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl ToCstr for u32 {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl ToCstr for u64 {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl ToCstr for f32 {
    fn cstr(&self) -> Cstr {
        format!("{self:.2}")
    }
}

impl ToCstr for f64 {
    fn cstr(&self) -> Cstr {
        format!("{self:.2}")
    }
}

impl ToCstr for i32 {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }

    fn cstr_expanded(&self) -> Cstr {
        match self.signum() {
            1 => format!("+{self}").cstr_c(GREEN),
            -1 => format!("{self}").cstr_c(RED),
            _ => format!("{self}"),
        }
    }
}

impl ToCstr for bool {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl ToCstr for Vec2 {
    fn cstr(&self) -> Cstr {
        format!("{}×{}", self.x as i32, self.y as i32)
    }
}

impl ToCstr for Color {
    fn cstr(&self) -> Cstr {
        self.c32().cstr()
    }
}

impl ToCstr for Color32 {
    fn cstr(&self) -> Cstr {
        self.to_hex().cstr_c(*self)
    }
}

impl ToCstr for HexColor {
    fn cstr(&self) -> Cstr {
        let s = &self.0;
        format!("[{s} {s}]")
    }
}

impl ToCstr for VarName {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(self.color())
    }
}

impl ToCstr for VarValue {
    fn cstr(&self) -> Cstr {
        match self {
            _ => self.to_string().cstr(),
        }
    }

    fn cstr_expanded(&self) -> Cstr {
        format!("[tw [s {}]] [th {}]", self.as_ref().cstr(), self.cstr())
    }
}

impl ToCstr for Expression {
    fn cstr(&self) -> Cstr {
        match self {
            Self::r#if(..) => "if",
            Self::r#mod(..) => "mod",
            _ => self.as_ref(),
        }
        .cstr_c(self.color())
    }
}

impl ToCstr for PainterAction {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }

    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            PainterAction::circle(x)
            | PainterAction::rectangle(x)
            | PainterAction::text(x)
            | PainterAction::hollow(x)
            | PainterAction::translate(x)
            | PainterAction::rotate(x)
            | PainterAction::scale_mesh(x)
            | PainterAction::scale_rect(x)
            | PainterAction::alpha(x)
            | PainterAction::feathering(x)
            | PainterAction::color(x) => x.cstr_expanded(),
            PainterAction::curve {
                thickness,
                curvature,
            } => format!(
                "{}, {}",
                thickness.cstr_expanded(),
                curvature.cstr_expanded()
            ),
            PainterAction::repeat(x, a) => format!("{}, {}", x.cstr_expanded(), a.cstr_expanded()),
            PainterAction::list(vec) => vec.into_iter().map(|a| a.cstr_expanded()).join(", "),
            PainterAction::if_ok(expr, actions) => {
                let actions_str = actions.into_iter().map(|a| a.cstr_expanded()).join(", ");
                format!("{}, [{}]", expr.cstr_expanded(), actions_str)
            }
            PainterAction::exit => Default::default(),
            PainterAction::paint => Default::default(),
        };
        format!("{}({inner})", self.cstr())
    }
}

impl ToCstr for Trigger {
    fn cstr(&self) -> Cstr {
        let mut s = self.as_ref().to_owned().cstr_c(self.color());
        match self {
            Trigger::ChangeStat(var) => {
                s += " ";
                s += &var.cstr();
            }
            Trigger::BattleStart
            | Trigger::TurnEnd
            | Trigger::BeforeDeath
            | Trigger::BeforeStrike
            | Trigger::AfterStrike
            | Trigger::DamageTaken
            | Trigger::DamageDealt
            | Trigger::StatusApplied
            | Trigger::StatusGained
            | Trigger::ChangeOutgoingDamage
            | Trigger::ChangeIncomingDamage
            | Trigger::AllyDeath => {}
        }
        s
    }
}

impl ToCstr for Action {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned().cstr_c(self.color())
    }
}

impl ToCstr for Event {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl ToCstr for NodeError {
    fn cstr(&self) -> Cstr {
        format!("[red {self}]").cstr_s(CstrStyle::Small)
    }
}

impl ToCstr for Reaction {
    fn cstr(&self) -> Cstr {
        let mut s = String::new();
        s += &self.trigger.cstr();
        s += "\n";
        s += &self
            .actions
            .iter()
            .map(|a| format!("  {}", a.cstr()))
            .join("\n");
        s
    }
}

impl ToCstr for Material {
    fn cstr(&self) -> Cstr {
        format!("Material([th {}])", self.0.len())
    }

    fn cstr_expanded(&self) -> Cstr {
        self.0.iter().map(|a| a.cstr()).join("\n")
    }
}

impl ToCstr for NodeKind {
    fn cstr(&self) -> Cstr {
        if matches!(self, NodeKind::None) {
            return "All".to_owned();
        }
        self.as_ref()
            .split_at(1)
            .1
            .cstr()
            .to_case(convert_case::Case::Title)
    }
}
