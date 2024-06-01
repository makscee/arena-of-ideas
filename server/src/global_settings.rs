use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub shop_slots_min: u32,
    pub shop_slots_max: u32,
    pub shop_slots_per_round: f32,
    pub team_slots: u32,
}

impl GlobalSettings {
    pub fn init() -> Result<(), String> {
        GlobalSettings::delete_by_always_zero(&0);
        GlobalSettings::insert(GlobalSettings {
            always_zero: 0,
            shop_slots_min: 3,
            shop_slots_max: 6,
            shop_slots_per_round: 0.34,
            team_slots: 7,
        })?;
        Ok(())
    }

    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
}
