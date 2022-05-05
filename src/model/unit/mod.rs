use super::*;

mod ai;
mod template;

pub use ai::*;
pub use template::*;

pub type UnitType = String;
pub type Tier = std::num::NonZeroUsize;

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
    pub base_damage: Health,
    pub face_dir: Vec2<Coord>,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub action: ActionProperties,
    pub radius: Coord,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub ability_cooldown: Option<Time>,
    pub alliances: HashSet<Alliance>,
    pub next_action_modifiers: Vec<Modifier>,
    #[serde(skip)]
    pub render: RenderConfig,
    pub last_action_time: Time,
    pub last_injure_time: Time,
    pub random_number: R32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitStat {
    MaxHealth,
    Radius,
    BaseDamage,
}

impl Unit {
    pub fn stat(&self, stat: UnitStat) -> R32 {
        match stat {
            UnitStat::MaxHealth => self.max_hp,
            UnitStat::Radius => self.radius,
            UnitStat::BaseDamage => self.base_damage,
        }
    }
    pub fn stat_mut(&mut self, stat: UnitStat) -> &mut R32 {
        match stat {
            UnitStat::MaxHealth => &mut self.max_hp,
            UnitStat::Radius => &mut self.radius,
            UnitStat::BaseDamage => &mut self.base_damage,
        }
    }
}
