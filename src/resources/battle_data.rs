#[derive(Default)]
pub struct BattleData {
    pub score_entity: Option<legion::Entity>,
    pub team_names_entitities: Option<(legion::Entity, legion::Entity)>,
    pub last_difficulty: usize,
}
