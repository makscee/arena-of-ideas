use super::*;
use once_cell::sync::OnceCell;

pub static UNIT_REP: OnceCell<Representation> = OnceCell::new();
pub static HERO_REP: OnceCell<Representation> = OnceCell::new();

pub fn unit_rep() -> &'static Representation {
    UNIT_REP.get().unwrap()
}
pub fn hero_rep() -> &'static Representation {
    HERO_REP.get().unwrap()
}
