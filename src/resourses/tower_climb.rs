use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TowerClimb {
    pub team: PackedTeam,
    pub owner_team: Option<PackedTeam>,
    pub defeated: usize,
    pub shop: ShopState,
}
