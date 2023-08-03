use super::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SaveData {
    pub team: Option<PackedTeam>,
    pub level: usize,
    pub ladder: Vec<ReplicatedTeam>,
}
