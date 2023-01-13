use super::*;

#[derive(Clone, HasId)]
pub struct UnitRender {
    pub id: Id,
    pub faction: Faction,
    pub position: Position,
    pub layers: Vec<ShaderProgram>,
}

impl UnitRender {
    pub fn new_from_unit(unit: Unit) -> Self {
        Self {
            id: unit.id,
            faction: unit.faction,
            position: Vec2::ZERO,
            layers: default(),
        }
    }
}
