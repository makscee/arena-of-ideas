use super::*;

#[derive(Debug, Clone)]
pub struct CorpseComponent {
    pub killer: legion::Entity,
}

impl CorpseComponent {
    pub fn from_unit(killer: legion::Entity) -> Self {
        Self { killer }
    }
}
