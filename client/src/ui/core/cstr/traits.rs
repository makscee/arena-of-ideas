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

    fn cstr_expanded(&self) -> Cstr {
        if let Some(description) = Descriptions::get(self) {
            return description.clone();
        }
        let inner = match self {
            Expression::one
            | Expression::zero
            | Expression::pi
            | Expression::pi2
            | Expression::gt
            | Expression::unit_size
            | Expression::all_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::all_enemy_units
            | Expression::owner
            | Expression::target => String::default(),
            Expression::var(v) | Expression::var_sum(v) => v.cstr(),
            Expression::value(v) => v.cstr(),
            Expression::string(v) => v.to_owned(),
            Expression::f32(v) | Expression::f32_slider(v) => v.cstr(),
            Expression::i32(v) => v.cstr(),
            Expression::bool(v) => v.cstr(),
            Expression::vec2(x, y) => vec2(*x, *y).cstr(),
            Expression::color(c) => match c.try_c32() {
                Ok(color) => c.cstr_c(color),
                Err(e) => format!("{c} [s {e:?}]",).cstr_c(RED),
            },
            Expression::lua_i32(code) | Expression::lua_f32(code) => code.cstr(),
            Expression::sin(x)
            | Expression::cos(x)
            | Expression::even(x)
            | Expression::abs(x)
            | Expression::floor(x)
            | Expression::ceil(x)
            | Expression::fract(x)
            | Expression::unit_vec(x)
            | Expression::rand(x)
            | Expression::random_unit(x)
            | Expression::to_f32(x)
            | Expression::sqr(x)
            | Expression::neg(x) => x.cstr_expanded(),
            Expression::str_macro(a, b)
            | Expression::vec2_ee(a, b)
            | Expression::sum(a, b)
            | Expression::sub(a, b)
            | Expression::mul(a, b)
            | Expression::div(a, b)
            | Expression::max(a, b)
            | Expression::min(a, b)
            | Expression::r#mod(a, b)
            | Expression::and(a, b)
            | Expression::or(a, b)
            | Expression::equals(a, b)
            | Expression::greater_then(a, b)
            | Expression::less_then(a, b)
            | Expression::fallback(a, b) => format!("{}, {}", a.cstr_expanded(), b.cstr_expanded()),
            Expression::oklch(a, b, c) | Expression::r#if(a, b, c) => format!(
                "{}, {}, {}",
                a.cstr_expanded(),
                b.cstr_expanded(),
                c.cstr_expanded()
            ),
            Expression::state_var(x, v) => format!("{}({})", x.cstr_expanded(), v.cstr_expanded()),
        };
        if inner.is_empty() {
            self.cstr()
        } else {
            format!("{}[tl (]{inner}[tl )]", self.cstr())
        }
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
            PainterAction::paint => Default::default(),
        };
        format!("{}({inner})", self.cstr())
    }
}

impl ToCstr for Trigger {
    fn cstr(&self) -> Cstr {
        let mut s = self.as_ref().to_owned().cstr_c(self.color());
        match self {
            Trigger::ChangeStat(var_name) => {
                s += " ";
                s += &var_name.cstr();
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

impl ToCstr for MagicType {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(self.color())
    }
}

impl ToCstr for Action {
    fn cstr(&self) -> Cstr {
        let inner_x = <Self as Injector<Expression>>::get_inner(self);
        let inner_a = <Self as Injector<Action>>::get_inner(self);
        let s = self.as_ref().to_owned().cstr_c(self.color());
        if !inner_x.is_empty() || !inner_a.is_empty() {
            let inner = inner_x
                .into_iter()
                .map(|x| x.cstr_expanded())
                .chain(inner_a.into_iter().map(|a| a.cstr_expanded()))
                .join(", ");
            format!("{s}[tl (]{inner}[tl )]")
        } else {
            s
        }
    }
}

impl ToCstr for Event {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl ToCstr for NodeError {
    fn cstr(&self) -> Cstr {
        format!("{self}").cstr_cs(RED, CstrStyle::Small)
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
