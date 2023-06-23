use super::*;

#[derive(Default)]
pub struct BattleData {
    pub team_names_entitities: Option<(legion::Entity, legion::Entity)>,
    pub last_difficulty: usize,
    pub last_round: usize,
    pub last_score: usize,
    pub total_score: usize,
    pub team_queue: HashMap<Faction, VecDeque<PackedTeam>>,
}
