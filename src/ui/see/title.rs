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

impl SFnTitle for NActionAbility {
    fn see_title_cstr(&self) -> Cstr {
        self.ability_name.cstr()
    }
}

impl SFnTitle for NStatusAbility {
    fn see_title_cstr(&self) -> Cstr {
        self.status_name.cstr()
    }
}
