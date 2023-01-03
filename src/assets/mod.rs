use super::*;

mod ability;
mod clan;
mod round;
mod unit_template;

use ability::*;
use clan::*;
use round::*;
use unit_template::*;

pub struct Assets {
    pub units: Vec<UnitTemplate>,
    pub clans: Vec<Clan>,
    pub rounds: Vec<Round>,
}
