use super::*;

/// Title trait for types that can be displayed as a colored string title.
/// This trait leverages SFnCstr for the actual string formatting.
pub trait SFnTitle {
    fn cstr_title(&self) -> Cstr;
}

// Basic type implementations that delegate to SFnCstr
impl SFnTitle for String {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for &str {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for &String {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for NodeKind {
    fn cstr_title(&self) -> Cstr {
        // Use the SFnCstr implementation which already handles this
        self.cstr()
    }
}

impl SFnTitle for NUnit {
    fn cstr_title(&self) -> Cstr {
        self.unit_name.cstr()
    }
}

impl SFnTitle for NHouse {
    fn cstr_title(&self) -> Cstr {
        self.house_name.cstr()
    }
}

impl SFnTitle for NAbilityMagic {
    fn cstr_title(&self) -> Cstr {
        self.ability_name.cstr()
    }
}

impl SFnTitle for NStatusMagic {
    fn cstr_title(&self) -> Cstr {
        self.status_name.cstr()
    }
}

impl SFnTitle for NFusion {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NHouseColor {
    fn cstr_title(&self) -> Cstr {
        self.color.cstr()
    }
}

impl SFnTitle for NAbilityDescription {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NAbilityEffect {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NStatusDescription {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NStatusBehavior {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NStatusRepresentation {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitStats {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for NUnitState {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitDescription {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitRepresentation {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitBehavior {
    fn cstr_title(&self) -> Cstr {
        self.kind().cstr()
    }
}

// These types already have SFnCstr implementations,
// so their title is just their colored string representation
impl SFnTitle for Expression {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for Action {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for Reaction {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for PainterAction {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for VarName {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for VarValue {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for i32 {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for f32 {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for bool {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for HexColor {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for Vec2 {
    fn cstr_title(&self) -> Cstr {
        self.cstr()
    }
}
