use crate::assets::UnitTemplate;

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitStats {
    pub health: i32,
    pub attack: i32,
    pub stacks: i32,
}

impl UnitStats {
    pub fn new(template: &UnitTemplate) -> Self {
        Self {
            health: template.health,
            attack: template.attack,
            stacks: 1,
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
            UnitStat::Level => &mut self.stacks,
        }
    }

    pub fn do_stack(&mut self, stats: UnitStats) -> bool {
        if self.level() < MAX_LEVEL {
            self.stacks += stats.stacks;
            self.merge_unit(stats);
            return true;
        }
        false
    }

    pub fn level(&self) -> i32 {
        self.stacks / STACKS_PER_LVL + 1
    }

    pub fn stacks_left_to_level(&self) -> i32 {
        STACKS_PER_LVL - self.stacks % STACKS_PER_LVL
    }

    fn merge_unit(&mut self, stats: UnitStats) {
        //Add +1/+1 instead of merge stats
        self.health += stats.stacks;
        self.attack += stats.stacks;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitStat {
    Health,
    Attack,
    Level,
}

impl fmt::Display for UnitStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
