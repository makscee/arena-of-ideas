use super::*;

mod ai;
mod template;
mod trigger;

pub use ai::*;
pub use template::*;
pub use trigger::*;

pub type UnitType = String;

#[derive(Serialize, Deserialize, Clone)]
pub enum ActionState {
    None,
    Start { time: Time, target: Id },
    Cooldown { time: Time },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActionProperties {
    pub cooldown: Time,
    pub animation_delay: Time,
    pub radius: Coord,
    #[serde(default)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub unit_type: UnitType,
    pub spawn_animation_time_left: Option<Time>,
    pub attached_statuses: Vec<Status>,
    pub all_statuses: Vec<Status>,
    pub faction: Faction,
    pub action_state: ActionState,
    pub hp: Health,
    pub max_hp: Health,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub action: ActionProperties,
    pub size: Coord,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub ability_cooldown: Option<Time>,
    pub triggers: Vec<UnitTrigger>,
    pub alliances: HashSet<Alliance>,
    #[serde(skip)]
    pub render: RenderMode,
}

impl Unit {
    pub fn radius(&self) -> Coord {
        self.size / Coord::new(2.0)
    }
}
