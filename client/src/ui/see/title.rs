use super::*;

pub trait SFnTitle {
    fn cstr_title(&self) -> Cstr;
}

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
        if matches!(self, raw_nodes::NodeKind::None) {
            return "All".to_owned();
        }
        self.as_ref()
            .split_at(1)
            .1
            .cstr()
            .to_case(convert_case::Case::Title)
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
