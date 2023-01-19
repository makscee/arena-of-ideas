use super::*;

#[derive(Clone, HasId)]
pub struct UnitRender {
    pub id: Id,
    pub faction: Faction,
    pub position: Position,
    pub stats: UnitStats,
    pub layers: Vec<ShaderProgram>,
}

impl UnitRender {
    pub fn new_from_unit(unit: &Unit) -> Self {
        Self {
            id: unit.id,
            faction: unit.faction.clone(),
            position: Vec2::ZERO,
            stats: unit.stats.clone(),
            layers: default(),
        }
    }
}
