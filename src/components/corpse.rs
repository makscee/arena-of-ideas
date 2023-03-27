use super::*;

#[derive(Debug, Clone)]
pub struct CorpseComponent {
    pub faction: Faction,
    pub killer: legion::Entity,
}
