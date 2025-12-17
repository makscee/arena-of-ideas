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

impl ToCstr for Trigger {
    fn cstr(&self) -> Cstr {
        let mut s = match self {
            Trigger::Any(triggers) => {
                let trigger_strs = triggers
                    .iter()
                    .map(|t| t.cstr())
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("Any({})", trigger_strs).cstr()
            }
            _ => self.as_ref().to_owned().cstr_c(self.color()),
        };
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
            | Trigger::AllyDeath
            | Trigger::Any(_) => {}
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

impl ToCstr for Target {
    fn cstr(&self) -> Cstr {
        match self {
            Target::List(targets) => {
                let target_strs = targets
                    .iter()
                    .map(|t| t.cstr())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("List({})", target_strs).cstr()
            }
            _ => self.as_ref().to_owned().cstr_c(self.color()),
        }
    }
}

impl ToCstr for Behavior {
    fn cstr(&self) -> Cstr {
        format!(
            "[b {}]: {}\n[s {}]",
            self.trigger.cstr(),
            self.effect.description,
            self.effect.actions.iter().map(|a| a.cstr()).join(" ")
        )
    }
}

impl ToCstr for Material {
    fn cstr(&self) -> Cstr {
        if self.0.code.is_empty() {
            "Material(empty)".to_string()
        } else {
            format!("Material({})", self.0.code.lines().count())
        }
    }

    fn cstr_expanded(&self) -> Cstr {
        self.0.code.clone()
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
