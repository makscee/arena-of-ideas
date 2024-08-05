use itertools::Itertools;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};

use self::base_unit::TBaseUnit;

use super::*;

#[derive(SpacetimeType, Clone)]
pub struct FusedUnit {
    pub id: u64,
    pub bases: Vec<String>,
    pub triggers: Vec<u32>,
    pub targets: Vec<u32>,
    pub effects: Vec<u32>,
    pub hp: i32,
    pub pwr: i32,
    pub lvl: u32,
    xp: u32,
}

impl PartialEq for FusedUnit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl FusedUnit {
    pub fn name(&self) -> String {
        self.bases.join("+")
    }
    pub fn new_id(mut self) -> Self {
        self.id = next_id();
        self
    }
    pub fn from_base(name: String, id: u64) -> Result<Self, String> {
        let base = TBaseUnit::filter_by_name(&name)
            .with_context_str(|| format!("Base unit not found {name}"))?;
        Ok(Self {
            bases: vec![name],
            triggers: vec![0],
            targets: vec![0],
            effects: vec![0],
            hp: base.hp,
            pwr: base.pwr,
            lvl: 0,
            xp: 0,
            id,
        })
    }
    fn roll_stat_mutation() -> i32 {
        let weights = [1, 10, 50, 10, 1];
        let stats = [-2, -1, 0, 1, 2];
        let index = WeightedIndex::new(&weights).unwrap();
        let index = index.sample(&mut thread_rng());
        stats[index]
    }
    pub fn mutate(mut self) -> Self {
        self.hp += Self::roll_stat_mutation();
        self.pwr += Self::roll_stat_mutation();
        self
    }
    pub fn fuse(a: &FusedUnit, b: &FusedUnit) -> Result<Vec<FusedUnit>, String> {
        if !Self::can_fuse(a, b) {
            return Err(format!("{} and {} can not be fused", a.name(), b.name()));
        }
        let mut options: Vec<FusedUnit> = Vec::new();
        let mut option = a.clone();
        option.bases.extend(b.bases.clone());
        option.hp = a.hp.max(b.hp);
        option.pwr = a.pwr.max(b.pwr);
        option.add_fuse_xp(b);

        let i = ((option.bases.len() - b.bases.len()) as u32)..option.bases.len() as u32;
        if !b.triggers.is_empty() {
            let mut option = option.clone().new_id();
            option.triggers.extend(i.clone());
            options.push(option);
        }
        if !b.targets.is_empty() {
            let mut option = option.clone().new_id();
            option.targets.extend(i.clone());
            options.push(option);
        }
        if !b.effects.is_empty() {
            let mut option = option.clone().new_id();
            option.effects.extend(i.clone());
            options.push(option);
        }
        Ok(options)
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
    pub fn can_fuse(a: &FusedUnit, b: &FusedUnit) -> bool {
        if !a.fusible() || !b.fusible() {
            return false;
        }
        let a = a.get_houses();
        let b = b.get_houses();
        !a.into_iter().any(|h| b.contains(&h))
    }
    pub fn fusible(&self) -> bool {
        self.lvl >= (self.bases.len() as u32)
    }
    pub fn add_xp(&mut self, xp: u32) {
        self.xp += xp;
        while self.xp > self.lvl {
            self.add_lvl();
            self.xp -= self.lvl;
        }
    }
    pub fn add_lvl(&mut self) {
        self.lvl += 1;
        self.pwr += 1;
        self.hp += 1;
    }
    pub fn total_xp(&self) -> u32 {
        self.lvl * (self.lvl + 1) / 2 + self.xp
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
