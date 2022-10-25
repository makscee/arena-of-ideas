use super::*;
mod template;

pub use template::*;

pub type UnitType = String;
pub type Tier = u32;
pub const MAX_TIER: u32 = 5;
pub const MAX_LEVEL: i32 = 3;

#[derive(Clone)]
pub enum TurnPhase {
    None,
    PreStrike,
    Strike,
    PostStrike,
}

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub unit_type: UnitType,
    pub spawn_animation_time_left: Option<Time>,
    pub all_statuses: Vec<AttachedStatus>,
    pub active_auras: HashSet<Id>,
    pub modifier_targets: Vec<(EffectContext, ModifierTarget)>,
    /// Temporary flags that live for one frame
    pub flags: Vec<UnitStatFlag>,
    pub faction: Faction,
    /// These stats are temporary and are reset every tick.
    /// They are modified primarily by modifier statuses
    pub stats: UnitStats,
    /// Permanent stats remain for the whole game round
    pub permanent_stats: UnitStats,
    pub position: Position,
    #[serde(default)]
    pub action: Effect,
    pub clans: Vec<Clan>,
    pub render: UnitRenderConfig,
    pub random_number: R32,
    pub shop_unit: Box<Option<Unit>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitStats {
    pub health: i32,
    pub attack: i32,
    pub stack: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitRenderConfig {
    pub base_shader_config: ShaderConfig,
    pub clan_shader_configs: Vec<Vec<ShaderConfig>>,
    pub radius: R32,
    pub render_position: Vec2<R32>,
    pub last_action_time: Time,
    pub last_injure_time: Time,
    pub last_heal_time: Time,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitStat {
    Health,
    Attack,
    Level,
}

impl Unit {
    pub fn new(
        template: &UnitTemplate,
        id: Id,
        unit_type: UnitType,
        faction: Faction,
        position: Position,
        statuses: &Statuses,
    ) -> Self {
        Self {
            id,
            unit_type,
            spawn_animation_time_left: Some(template.spawn_animation_time),
            all_statuses: template
                .statuses
                .iter()
                .map(|status| status.get(statuses).clone().attach(Some(id), Some(id), id))
                .collect(),
            active_auras: default(),
            modifier_targets: vec![],
            flags: vec![],
            faction,
            stats: UnitStats::new(template),
            permanent_stats: UnitStats::new(template),
            position,
            action: template.action.clone(),
            render: UnitRenderConfig::new(template),
            clans: template.clans.clone(),
            random_number: r32(global_rng().gen_range(0.0..=1.0)),
            shop_unit: Box::new(None),
        }
    }
    pub fn level_up(&mut self, unit: Unit) -> bool {
        if unit.unit_type == self.unit_type {
            if self.stats.level_up(unit.stats)
                && self.permanent_stats.level_up(unit.permanent_stats)
            {
                return true;
            }
        }
        false
    }
}

impl UnitStats {
    pub fn new(template: &UnitTemplate) -> Self {
        Self {
            health: template.health,
            attack: template.attack,
            stack: 1,
        }
    }

    pub fn get(&self, stat: UnitStat) -> i32 {
        match stat {
            UnitStat::Health => self.health,
            UnitStat::Attack => self.attack,
            UnitStat::Level => self.level(),
        }
    }
    pub fn get_mut(&mut self, stat: UnitStat) -> &mut i32 {
        match stat {
            UnitStat::Health => &mut self.health,
            UnitStat::Attack => &mut self.attack,
            UnitStat::Level => &mut self.stack,
        }
    }

    pub fn level_up(&mut self, stats: UnitStats) -> bool {
        if self.level() < MAX_LEVEL {
            self.stack += stats.stack;
            self.merge_unit(stats);
            return true;
        }
        false
    }

    fn level(&self) -> i32 {
        let mut stack = self.stack;
        let mut level = 1;
        for i in 1..MAX_LEVEL + 1 {
            stack -= i;
            level = i;
            if stack <= 0 {
                break;
            }
        }

        level
    }

    fn merge_unit(&mut self, stats: UnitStats) {
        self.health += stats.health;
        self.attack += stats.attack;
    }
}

impl UnitRenderConfig {
    pub fn new(template: &UnitTemplate) -> Self {
        Self {
            base_shader_config: template.render_config.clone(),
            clan_shader_configs: template
                .clan_renders
                .iter()
                .map(|render| render.clone())
                .collect(),
            radius: template.radius,
            render_position: Vec2::ZERO,
            last_action_time: Time::new(0.0),
            last_injure_time: Time::new(0.0),
            last_heal_time: Time::new(0.0),
        }
    }
}
