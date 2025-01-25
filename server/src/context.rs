use super::*;

pub struct Context<'a> {
    pub rc: &'a ReducerContext,
    pub player_id: u64,
}
