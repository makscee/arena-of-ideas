use macro_server::*;
use schema::*;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
}

impl Hero {
    pub fn new(name: String) -> Self {
        Self {
            name,
            representation: None,
            mover: None,
        }
    }
}
impl Mover {
    pub fn new() -> Self {
        Self {
            target: Vec2::default(),
            from: Vec2::default(),
            start_ts: now_seconds(),
        }
    }
}
