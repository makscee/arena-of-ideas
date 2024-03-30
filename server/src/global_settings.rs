use super::*;

#[spacetimedb(table)]
pub struct GlobalSettings {
    #[unique]
    always_zero: u32,
    pub team_slots: u32,
    pub fatigue_start: u32,
}

impl GlobalSettings {
    pub fn init() -> Result<(), String> {
        GlobalSettings::delete_by_always_zero(&0);
        GlobalSettings::insert(GlobalSettings {
            always_zero: 0,
            team_slots: 7,
            fatigue_start: 20,
        })?;
        Ok(())
    }

    pub fn get() -> Self {
        GlobalSettings::filter_by_always_zero(&0).unwrap()
    }
}
