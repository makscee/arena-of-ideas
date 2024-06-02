use itertools::Itertools;

use self::base_unit::BaseUnit;

use super::*;

#[derive(SpacetimeType, Clone)]
pub struct FusedUnit {
    pub bases: Vec<String>,
    pub triggers: Vec<u32>,
    pub targets: Vec<u32>,
    pub effects: Vec<u32>,
    pub stacks: u32,
}

impl FusedUnit {
    pub fn from_base(name: String) -> Self {
        Self {
            bases: vec![name],
            triggers: vec![0],
            targets: vec![0],
            effects: vec![0],
            stacks: 1,
        }
    }
    pub fn get_bases(&self) -> Vec<BaseUnit> {
        BaseUnit::iter()
            .filter(|u| self.bases.contains(&u.name))
            .collect_vec()
    }
    pub fn get_houses(&self) -> Vec<String> {
        self.get_bases().into_iter().map(|u| u.house).collect_vec()
    }
}
