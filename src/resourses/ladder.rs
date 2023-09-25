use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Ladder {
    pub teams: Vec<PackedTeam>,
}
