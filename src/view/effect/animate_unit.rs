use super::*;

pub struct AnimateUnitVisualEffect {
    pub id: Id,
    pub from: Position,
    pub to: Position,
}

impl AnimateUnitVisualEffect {
    pub fn new(id: Id, from: Position, to: Position) -> Self {
        Self { id, from, to }
    }
}

impl VisualEffect for AnimateUnitVisualEffect {
    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        let unit = model
            .units
            .get_mut(&self.id)
            .expect(&format!("Attempt to animate absent unit#{}", self.id));
        unit.position += self.from + (self.to - self.from) * t;
    }
}
