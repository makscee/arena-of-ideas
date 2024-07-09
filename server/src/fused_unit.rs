use itertools::Itertools;

use self::base_unit::BaseUnit;

use super::*;

#[derive(SpacetimeType, Clone)]
pub struct FusedUnit {
    pub id: GID,
    pub bases: Vec<String>,
    pub triggers: Vec<u32>,
    pub targets: Vec<u32>,
    pub effects: Vec<u32>,
    pub stacks: u32,
}

impl FusedUnit {
    pub fn from_base(name: String, id: GID) -> Self {
        Self {
            bases: vec![name],
            triggers: vec![0],
            targets: vec![0],
            effects: vec![0],
            stacks: 1,
            id,
        }
    }
    pub fn get_bases(&self) -> Vec<BaseUnit> {
        self.bases
            .iter()
            .map(|b| BaseUnit::filter_by_name(b).unwrap())
            .collect_vec()
    }
    pub fn get_houses(&self) -> Vec<String> {
        self.get_bases().into_iter().map(|u| u.house).collect_vec()
    }
    pub fn can_stack(&self, name: &str) -> bool {
        if self.bases.len() == 1 && name.eq(&self.bases[0]) {
            return true;
        }
        false
    }
}
