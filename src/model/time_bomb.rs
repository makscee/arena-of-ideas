use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, HasId)]
pub struct TimeBomb {
    pub id: Id,
    pub position: Position,
    pub time: Time,
    pub caster: Option<Id>,
    pub effect: Effect,
}
