use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, HasId)]
pub struct TimeBomb {
    pub id: Id,
    pub position: Vec2<Coord>,
    pub time: Time,
    pub caster: Option<Id>,
    pub effect: Effect,
}
