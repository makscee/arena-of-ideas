use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub team_slots: u32,
    pub fatigue_start: u32,
    pub price_unit_buy_stack: f32,
    pub price_unit_sell: f32,
    pub price_unit_discount: f32,
    pub price_reroll: i32,
    pub shop_slots_min: u32,
    pub shop_slots_max: u32,
    pub shop_slots_per_round: f32,
    pub g_per_round_min: i32,
    pub g_per_round_max: i32,
    pub discount_chance: f64,
    pub season: u32,
    pub rarities: Rarities,
}

#[derive(SpacetimeType)]
pub struct Rarities {
    pub prices: Vec<i32>,
    pub weights_initial: Vec<i32>,
    pub weights_per_round: Vec<i32>,
}

impl GlobalSettings {
    pub fn init() -> Result<(), String> {
        GlobalSettings::delete_by_always_zero(&0);
        GlobalSettings::insert(GlobalSettings {
            always_zero: 0,
            team_slots: 7,
            fatigue_start: 20,
            price_unit_buy_stack: 0.75,
            price_unit_sell: 0.5,
            price_unit_discount: 0.5,
            price_reroll: 1,
            shop_slots_min: 3,
            shop_slots_max: 6,
            shop_slots_per_round: 0.34,
            g_per_round_min: 6,
            g_per_round_max: 14,
            discount_chance: 0.1,
            season: 0,
            rarities: Rarities {
                prices: [4, 6, 8, 10].into(),
                weights_initial: [100, 5, -10, -20].into(),
                weights_per_round: [0, 5, 5, 5].into(),
            },
        })?;
        Ok(())
    }

    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
}
