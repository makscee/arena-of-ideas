use super::*;

#[spacetimedb(table(public))]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub season: u32,
    pub arena: ArenaSettings,
    pub rarities: RaritySettings,
    pub battle: BattleSettings,
    pub craft_shards_cost: u32,
    pub create_team_cost: i64,
    pub meta: MetaSettings,
    pub ghost_unit: String,
    pub quest: QuestSettings,
}

impl GlobalSettings {
    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
    pub fn replace(self) {
        GlobalSettings::delete_by_always_zero(&0);
        let _ = GlobalSettings::insert(self);
    }
}

#[derive(SpacetimeType)]
pub struct BattleSettings {
    pub fatigue_start: u32,
    pub deafness_start: u32,
    pub deafness_per_turn: f64,
    pub summon_limit: u32,
}

#[derive(SpacetimeType)]
pub struct ArenaSettings {
    pub shop_slots: InflatingInt,
    pub g_income: InflatingInt,

    pub ranked_cost: i64,
    pub const_cost: i64,
    pub price_reroll: i32,
    pub sell_discount: i32,
    pub stack_discount: i32,
    pub team_slots: u32,
    pub lives_initial: u32,
    pub free_rerolls_initial: u32,
    pub free_rerolls_income: u32,
    pub initial_enemies_count: Vec<u32>,
}

#[derive(SpacetimeType)]
pub struct RaritySettings {
    pub prices: Vec<i32>,
    pub weights_initial: Vec<i32>,
    pub weights_per_floor: Vec<i32>,
    pub lootbox_weights: Vec<i32>,
}

#[derive(SpacetimeType)]
pub struct MetaSettings {
    pub price_lootbox: i64,
    pub price_shard: i64,
    pub shop_shard_slots: u32,
    pub balance_vote_reward: i64,
}

#[derive(SpacetimeType)]
pub struct QuestSettings {
    pub daily_options: u32,
    pub daily_limit: u32,
}
