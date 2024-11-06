use super::*;

#[spacetimedb::table(name = status)]
pub struct TStatus {
    #[primary_key]
    pub name: String,
    pub description: String,
    pub polarity: i8,
    pub trigger: String,
}
