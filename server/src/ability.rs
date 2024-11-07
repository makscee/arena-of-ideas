#[spacetimedb::table(public, name = ability)]
pub struct TAbility {
    #[primary_key]
    pub name: String,
    pub description: String,
    pub effect: String,
}
