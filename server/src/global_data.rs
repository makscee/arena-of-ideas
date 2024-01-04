use super::*;

#[spacetimedb(table)]
pub struct GlobalData {
    #[unique]
    always_zero: u32,
    pub game_version: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init() -> Result<(), String> {
        GlobalData::insert(GlobalData {
            always_zero: 0,
            game_version: VERSION.to_owned(),
        })?;
        Ok(())
    }
}
