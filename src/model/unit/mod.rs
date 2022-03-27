use super::*;

mod ai;
mod template;

pub use ai::*;
pub use template::*;

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
    pub range: Coord,
    #[serde(default)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub unit_type: UnitType,
    pub spawn_animation_time_left: Option<Time>,
    pub attached_statuses: Vec<AttachedStatus>,
    pub all_statuses: Vec<Status>,
    pub faction: Faction,
    pub action_state: ActionState,
    pub health: Health,
    pub max_hp: Health,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub action: ActionProperties,
    pub radius: Coord,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub ability_cooldown: Option<Time>,
    pub alliances: HashSet<Alliance>,
    #[serde(skip)]
    pub render: RenderMode,
}
