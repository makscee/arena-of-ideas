use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub team_slots: u32,
    pub fatigue_start: u32,
    pub price_unit_buy: i64,
    pub price_unit_buy_stack: i64,
    pub price_unit_sell: i64,
    pub price_reroll: i64,
    pub shop_slots_min: u32,
    pub shop_slots_max: u32,
    pub shop_slots_per_round: f32,
    pub g_per_round_min: i64,
    pub g_per_round_max: i64,
}

impl GlobalSettings {
    pub fn init() -> Result<(), String> {
        GlobalSettings::delete_by_always_zero(&0);
        GlobalSettings::insert(GlobalSettings {
            always_zero: 0,
            team_slots: 7,
            fatigue_start: 20,
            price_unit_buy: 4,
            price_unit_buy_stack: 3,
            price_unit_sell: 2,
            price_reroll: 1,
            shop_slots_min: 3,
            shop_slots_max: 6,
            shop_slots_per_round: 0.34,
            g_per_round_min: 5,
            g_per_round_max: 9,
        })?;
        Ok(())
    }

    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
}
