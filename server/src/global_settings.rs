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

#[spacetimedb(reducer)]
fn upload_settings(ctx: ReducerContext, data: GlobalSettings) -> Result<(), String> {
    GlobalSettings::insert(data)?;
    Ok(())
}

impl GlobalSettings {
    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
}
