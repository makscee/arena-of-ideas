use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LadderClimb {
    pub team: PackedTeam,
    pub levels: Vec<String>,
    pub defeated: usize,
    pub shop: ShopState,
}
