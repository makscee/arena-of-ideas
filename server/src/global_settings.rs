use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub shop_slots_min: u32,
    pub shop_slots_max: u32,
    pub shop_slots_per_round: f32,
    pub shop_g_start: i32,
    pub shop_g_income_min: i32,
    pub shop_g_income_max: i32,
    pub shop_g_income_per_round: i32,
    pub shop_price_reroll: i32,
    pub shop_price_unit: i32,
    pub team_slots: u32,
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
