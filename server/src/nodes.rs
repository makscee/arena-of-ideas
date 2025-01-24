use macro_server::*;
use schema::*;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn id(&self) -> Option<u64>;
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
    fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self>;
    fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>);
    fn from_table(ctx: &ReducerContext, domain: NodeDomain, id: u64) -> Option<Self>;
    fn to_table(self, ctx: &ReducerContext, domain: NodeDomain, id: Option<u64>, parent: u64);
}

impl Hero {
    pub fn new(name: String) -> Self {
        Self { name, ..default() }
    }
}
impl Mover {
    pub fn new() -> Self {
        Self {
            start_ts: now_seconds(),
            ..default()
        }
    }
}
