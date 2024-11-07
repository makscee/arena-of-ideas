use itertools::Itertools;
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = base_unit)]
pub struct TBaseUnit {
    #[primary_key]
    pub name: String,
    pub pwr: i32,
    pub hp: i32,
    pub rarity: u8,
    pub house: String,
    pub pool: UnitPool,
    pub triggers: Vec<String>,
    pub targets: Vec<String>,
    pub effects: Vec<String>,
    pub representation: String,
}

#[derive(SpacetimeType, Clone, Copy, PartialEq, Eq)]
pub enum UnitPool {
    Game,
    Summon,
}

impl TBaseUnit {
    pub fn get_random(
        ctx: &ReducerContext,
        houses: &Vec<String>,
        weights: &Vec<i32>,
        rng: &mut impl Rng,
    ) -> Self {
        let mut units = ctx
            .db
            .base_unit()
            .iter()
            .filter(|u| {
                u.pool == UnitPool::Game && (houses.is_empty() || houses.contains(&u.house))
            })
            .collect_vec();
        let dist = WeightedIndex::new(units.iter().map(|u| weights[u.rarity as usize])).unwrap();
        units.remove(dist.sample(rng))
    }
    pub fn get_random_for_lootbox(ctx: &ReducerContext, houses: &Vec<String>) -> Self {
        Self::get_random(
            ctx,
            houses,
            &GlobalSettings::get(ctx).rarities.lootbox_weights,
            &mut ctx.rng(),
        )
    }
    pub fn into_fused(self, ctx: &ReducerContext) -> FusedUnit {
        FusedUnit::from_base(self, next_id(ctx))
    }
}
