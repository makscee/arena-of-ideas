use macro_server::*;
use schema::*;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
}
