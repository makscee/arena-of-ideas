use itertools::Itertools;

use self::base_unit::TBaseUnit;

use super::*;

#[derive(SpacetimeType, Clone)]
pub struct FusedUnit {
    pub id: GID,
    pub bases: Vec<String>,
    pub triggers: Vec<u32>,
    pub targets: Vec<u32>,
    pub effects: Vec<u32>,
    pub lvl: u32,
    xp: u32,
}

impl FusedUnit {
    pub fn name(&self) -> String {
        self.bases.join("+")
    }
    pub fn new_id(mut self) -> Self {
        self.id = next_id();
        self
    }
    pub fn from_base(name: String, id: GID) -> Self {
        Self {
            bases: vec![name],
            triggers: vec![0],
            targets: vec![0],
            effects: vec![0],
            lvl: 1,
            xp: 0,
            id,
        }
    }
    pub fn get_bases(&self) -> Vec<TBaseUnit> {
        self.bases
            .iter()
            .map(|b| TBaseUnit::filter_by_name(b).unwrap())
            .collect_vec()
    }
    pub fn get_houses(&self) -> Vec<String> {
        self.get_bases().into_iter().map(|u| u.house).collect_vec()
    }
    pub fn can_stack(&self, name: &String) -> bool {
        if self.bases.len() == 1 {
            return name.eq(&self.bases[0]);
        } else {
            self.get_houses()
                .contains(&TBaseUnit::filter_by_name(name).unwrap().house)
        }
    }
    pub fn can_stack_fused(&self, unit: &FusedUnit) -> bool {
        unit.bases.len() == 1 && self.can_stack(&unit.bases[0])
    }
    pub fn can_fuse_source(&self, source: &FusedUnit) -> bool {
        let houses = self.get_houses();
        source.bases.len() == 1
            && source.lvl > 1
            && !houses.contains(&source.get_houses()[0])
            && houses.len() < self.lvl as usize
    }
    pub fn add_xp(&mut self, xp: u32) {
        self.xp += xp;
        while self.xp >= self.lvl {
            self.xp -= self.lvl;
            self.lvl += 1;
        }
    }
    pub fn total_xp(&self) -> u32 {
        (self.lvl - 1) * self.lvl / 2 + self.xp
    }
    pub fn add_fuse_xp(&mut self, source: &FusedUnit) {
        self.add_xp(source.total_xp() - 1);
    }
    pub fn rarity(&self) -> i8 {
        self.get_bases()
            .into_iter()
            .map(|u| u.rarity)
            .max()
            .unwrap_or_default()
    }
}
