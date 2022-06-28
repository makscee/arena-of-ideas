use super::*;

mod ai;
mod template;

pub use ai::*;
pub use template::*;

pub type UnitType = String;
pub type Tier = u32;

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
    pub all_statuses: Vec<AttachedStatus>,
    /// Temporary flags that live for one frame
    pub flags: Vec<UnitStatFlag>,
    pub faction: Faction,
    pub action_state: ActionState,
    /// These stats are temporary and are reset every tick.
    /// They are modified primarily by modifier statuses
    pub stats: UnitStats,
    /// Permanent stats remain for the whole game round
    pub permanent_stats: UnitStats,
    pub health: Health,
    pub face_dir: Vec2<Coord>,
    pub position: Vec2<Coord>,
    pub action: ActionProperties,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub ability_cooldown: Option<Time>,
    pub clans: HashSet<Clan>,
    pub next_action_modifiers: Vec<Modifier>,
    #[serde(skip)]
    pub render: RenderConfig,
    pub last_action_time: Time,
    pub last_injure_time: Time,
    pub random_number: R32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitStats {
    pub max_hp: Health,
    pub radius: Coord,
    pub base_damage: R32,
    pub armor: R32,
    pub armor_penetration: R32,
    pub crit_chance: R32,
    pub speed: Coord,
    pub action_speed: R32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitStat {
    MaxHealth,
    Radius,
    BaseDamage,
    Armor,
    ArmorPenetration,
    CritChance,
    Speed,
    ActionSpeed,
}

impl Unit {
    pub fn new(
        template: &UnitTemplate,
        next_id: &mut Id,
        unit_type: UnitType,
        faction: Faction,
        position: Vec2<Coord>,
        statuses: &Statuses,
    ) -> Self {
        let id = *next_id;
        *next_id += 1;
        Self {
            id,
            unit_type,
            spawn_animation_time_left: Some(template.spawn_animation_time),
            all_statuses: template
                .statuses
                .iter()
                .map(|status| {
                    status
                        .get(statuses)
                        .clone()
                        .attach(Some(id), Some(id), next_id)
                })
                .collect(),
            flags: vec![],
            faction,
            action_state: ActionState::None,
            stats: UnitStats::new(template),
            permanent_stats: UnitStats::new(template),
            health: template.health,
            face_dir: Vec2::ZERO,
            position,
            action: template.action.clone(),
            move_ai: template.move_ai,
            target_ai: template.target_ai,
            render: template.render_config.clone(),
            next_action_modifiers: Vec::new(),
            ability_cooldown: None,
            clans: template.clans.clone(),
            last_action_time: Time::new(0.0),
            last_injure_time: Time::new(0.0),
            random_number: r32(global_rng().gen_range(0.0..=1.0)),
        }
    }
}

impl UnitStats {
    pub fn new(template: &UnitTemplate) -> Self {
        Self {
            max_hp: template.health,
            base_damage: template.base_damage,
            armor: template.armor,
            armor_penetration: template.armor_penetration,
            crit_chance: template.crit_chance,
            action_speed: template.action_speed,
            speed: template.speed,
            radius: template.radius,
        }
    }

    pub fn get(&self, stat: UnitStat) -> R32 {
        match stat {
            UnitStat::MaxHealth => self.max_hp,
            UnitStat::Radius => self.radius,
            UnitStat::BaseDamage => self.base_damage,
            UnitStat::Armor => self.armor,
            UnitStat::ArmorPenetration => self.armor_penetration,
            UnitStat::CritChance => self.crit_chance,
            UnitStat::ActionSpeed => self.action_speed,
            UnitStat::Speed => self.speed,
        }
    }
    pub fn get_mut(&mut self, stat: UnitStat) -> &mut R32 {
        match stat {
            UnitStat::MaxHealth => &mut self.max_hp,
            UnitStat::Radius => &mut self.radius,
            UnitStat::BaseDamage => &mut self.base_damage,
            UnitStat::Armor => &mut self.armor,
            UnitStat::ArmorPenetration => &mut self.armor_penetration,
            UnitStat::CritChance => &mut self.crit_chance,
            UnitStat::ActionSpeed => &mut self.action_speed,
            UnitStat::Speed => &mut self.speed,
        }
    }
}
