use super::*;

pub trait SFnTitle {
    fn see_title_cstr(&self) -> Cstr;
}

impl SFnTitle for String {
    fn see_title_cstr(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for &str {
    fn see_title_cstr(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for &String {
    fn see_title_cstr(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for NUnit {
    fn see_title_cstr(&self) -> Cstr {
        self.unit_name.cstr()
    }
}

impl SFnTitle for NHouse {
    fn see_title_cstr(&self) -> Cstr {
        self.house_name.cstr()
    }
}

impl SFnTitle for NAbilityMagic {
    fn see_title_cstr(&self) -> Cstr {
        self.ability_name.cstr()
    }
}

impl SFnTitle for NStatusMagic {
    fn see_title_cstr(&self) -> Cstr {
        self.status_name.cstr()
    }
}

impl SFnTitle for NFusion {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NHouseColor {
    fn see_title_cstr(&self) -> Cstr {
        self.color.cstr()
    }
}

impl SFnTitle for NAbilityDescription {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NAbilityEffect {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NStatusDescription {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NStatusBehavior {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NStatusRepresentation {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitStats {
    fn see_title_cstr(&self) -> Cstr {
        self.cstr()
    }
}

impl SFnTitle for NUnitState {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitDescription {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitRepresentation {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}

impl SFnTitle for NUnitBehavior {
    fn see_title_cstr(&self) -> Cstr {
        self.kind().cstr()
    }
}
