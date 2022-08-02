use super::*;
mod template;

pub use template::*;

pub type UnitType = String;
pub type Tier = u32;
pub const MAX_TIER: u32 = 5;

#[derive(Serialize, Deserialize, Clone)]
pub enum ActionState {
    None,
    Start { target: Id },
    Cooldown { time: Ticks },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActionProperties {
    #[serde(default)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub unit_type: UnitType,
    pub spawn_animation_time_left: Option<Time>,
    pub all_statuses: Vec<AttachedStatus>,
    pub modifier_targets: Vec<(EffectContext, ModifierTarget)>,
    /// Temporary flags that live for one frame
    pub flags: Vec<UnitStatFlag>,
    pub faction: Faction,
    pub action_state: ActionState,
    /// These stats are temporary and are reset every tick.
    /// They are modified primarily by modifier statuses
    pub stats: UnitStats,
    /// Permanent stats remain for the whole game round
    pub permanent_stats: UnitStats,
    pub face_dir: Vec2<R32>,
    pub position: Position,
    pub action: ActionProperties,
    pub cooldown: Ticks,
    pub range: Coord,
    pub ability_cooldown: Option<Time>,
    pub clans: Vec<Clan>,
    pub next_action_modifiers: Vec<Modifier>,
    #[serde(skip)]
    pub render: ShaderConfig,
    pub render_position: Vec2<R32>,
    pub last_action_time: Time,
    pub last_injure_time: Time,
    pub last_heal_time: Time,
    pub random_number: R32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitStats {
    pub max_hp: Health,
    pub health: Health,
    pub radius: R32,
    pub base_damage: R32,
    pub block: R32,
    pub crit_chance: R32,
    pub action_speed: R32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitStat {
    Health,
    MaxHealth,
    Radius,
    BaseDamage,
    Block,
    CritChance,
    ActionSpeed,
}

impl Unit {
    pub fn new(
        template: &UnitTemplate,
        next_id: &mut Id,
        unit_type: UnitType,
        faction: Faction,
        position: Position,
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
            modifier_targets: vec![],
            flags: vec![],
            range: template.range,
            cooldown: template.cooldown,
            faction,
            action_state: ActionState::Cooldown { time: 0 },
            stats: UnitStats::new(template),
            permanent_stats: UnitStats::new(template),
            face_dir: Vec2::ZERO,
            render_position: Vec2::ZERO,
            position,
            action: template.action.clone(),
            render: template.render_config.clone(),
            next_action_modifiers: Vec::new(),
            ability_cooldown: None,
            clans: template.clans.clone(),
            last_action_time: Time::new(0.0),
            last_injure_time: Time::new(0.0),
            last_heal_time: Time::new(0.0),
            random_number: r32(global_rng().gen_range(0.0..=1.0)),
        }
    }
}

impl UnitStats {
    pub fn new(template: &UnitTemplate) -> Self {
        Self {
            max_hp: template.health,
            health: template.health,
            base_damage: template.base_damage,
            block: template.block,
            crit_chance: template.crit_chance,
            action_speed: template.action_speed,
            radius: template.radius,
        }
    }

    pub fn get(&self, stat: UnitStat) -> R32 {
        match stat {
            UnitStat::Health => self.health,
            UnitStat::MaxHealth => self.max_hp,
            UnitStat::Radius => self.radius,
            UnitStat::BaseDamage => self.base_damage,
            UnitStat::Block => self.block,
            UnitStat::CritChance => self.crit_chance,
            UnitStat::ActionSpeed => self.action_speed,
        }
    }
    pub fn get_mut(&mut self, stat: UnitStat) -> &mut R32 {
        match stat {
            UnitStat::Health => &mut self.health,
            UnitStat::MaxHealth => &mut self.max_hp,
            UnitStat::Radius => &mut self.radius,
            UnitStat::BaseDamage => &mut self.base_damage,
            UnitStat::Block => &mut self.block,
            UnitStat::CritChance => &mut self.crit_chance,
            UnitStat::ActionSpeed => &mut self.action_speed,
        }
    }
}
