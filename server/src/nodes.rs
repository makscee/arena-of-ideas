use macro_server::*;
use schema::*;
use strum_macros::{Display, EnumIter};

macro_schema::nodes!();

pub trait Node: Default + Sized {
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
    fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self>;
    fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>);
    fn from_table(ctx: &ReducerContext, id: u64) -> Option<Self>;
}

impl Match {
    fn pack_table(ctx: &ReducerContext, id: u64) -> Option<Self> {
        let data = ctx
            .db
            .nodes_match()
            .key()
            .find(Self::kind_s().key(id))?
            .data;
        let mut d = Self::default();
        d.inject_data(&data);
        let children = ctx
            .db
            .nodes_relations()
            .parent()
            .filter(id)
            .map(|r| r.id)
            .collect_vec();
        d.team = Team::pack_table(ctx, id);
        d.shop_case = children
            .iter()
            .filter_map(|id| ShopCaseUnit::pack_table(ctx, *id))
            .collect();
        None
    }
}
impl Team {
    fn pack_table(ctx: &ReducerContext, id: u64) -> Option<Self> {
        None
    }
}
impl ShopCaseUnit {
    fn pack_table(ctx: &ReducerContext, id: u64) -> Option<Self> {
        None
    }
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
