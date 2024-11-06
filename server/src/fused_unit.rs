use base_unit::base_unit;
use itertools::Itertools;
use rand::distributions::{Distribution, WeightedIndex};

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
    pub hp_mutation: i32,
    pub pwr_mutation: i32,
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
    pub fn new_id(mut self, ctx: &ReducerContext) -> Self {
        self.id = next_id(ctx);
        self
    }
    pub fn from_base(base: TBaseUnit, id: u64) -> Self {
        Self {
            bases: vec![base.name],
            triggers: vec![0],
            targets: vec![0],
            effects: vec![0],
            hp: base.hp,
            pwr: base.pwr,
            hp_mutation: 0,
            pwr_mutation: 0,
            lvl: 1,
            xp: 0,
            id,
        }
    }
    pub fn from_base_name(ctx: &ReducerContext, name: String, id: u64) -> Result<Self, String> {
        let base = ctx
            .db
            .base_unit()
            .name()
            .find(&name)
            .with_context_str(|| format!("Base unit not found {name}"))?;
        Ok(Self::from_base(base, id))
    }
    fn roll_stat_mutation(ctx: &ReducerContext) -> i32 {
        let weights = [1, 10, 50, 10, 1];
        let stats = [-2, -1, 0, 1, 2];
        let index = WeightedIndex::new(&weights).unwrap();
        let index = index.sample(&mut ctx.rng());
        stats[index]
    }
    pub fn mutate(mut self, ctx: &ReducerContext) -> Self {
        self.hp_mutation = Self::roll_stat_mutation(ctx);
        self.hp = (self.hp + self.hp_mutation).max(1);
        self.pwr_mutation = Self::roll_stat_mutation(ctx);
        self.pwr = (self.pwr + self.pwr_mutation).max(0);
        self
    }
    pub fn fuse(
        a: &FusedUnit,
        b: &FusedUnit,
    ) -> Result<(FusedUnit, [Vec<u32>; 3], [Vec<u32>; 3], [Vec<u32>; 3]), String> {
        if !Self::can_fuse(a, b) {
            return Err(format!("{} and {} can not be fused", a.name(), b.name()));
        }
        let mut unit = a.clone();
        unit.bases.extend(b.bases.clone());
        unit.hp = a.hp.max(b.hp);
        unit.pwr = a.pwr.max(b.pwr);
        unit.add_fuse_xp(b);
        let offset = a.bases.len() as u32;
        let triggers = [
            a.triggers.clone(),
            a.triggers
                .iter()
                .copied()
                .chain(b.triggers.iter().map(|t| *t + offset))
                .collect_vec(),
            b.triggers
                .clone()
                .into_iter()
                .map(|t| t + offset)
                .collect_vec(),
        ];
        let targets = [
            a.targets.clone(),
            a.targets
                .iter()
                .copied()
                .chain(b.targets.iter().map(|t| *t + offset))
                .collect_vec(),
            b.targets
                .clone()
                .into_iter()
                .map(|t| t + offset)
                .collect_vec(),
        ];
        let effects = [
            a.effects.clone(),
            a.effects
                .iter()
                .copied()
                .chain(b.effects.iter().map(|t| *t + offset))
                .collect_vec(),
            b.effects
                .clone()
                .into_iter()
                .map(|t| t + offset)
                .collect_vec(),
        ];
        Ok((unit, triggers, targets, effects))
    }
    pub fn get_bases(&self, ctx: &ReducerContext) -> Vec<TBaseUnit> {
        self.bases
            .iter()
            .map(|b| ctx.db.base_unit().name().find(b).unwrap())
            .collect_vec()
    }
    pub fn get_houses(&self, ctx: &ReducerContext) -> Vec<String> {
        self.get_bases(ctx)
            .into_iter()
            .map(|u| u.house)
            .collect_vec()
    }
    pub fn can_stack(&self, ctx: &ReducerContext, name: &String) -> bool {
        if self.bases.len() == 1 {
            return name.eq(&self.bases[0]);
        } else {
            self.get_houses(ctx)
                .contains(&ctx.db.base_unit().name().find(name).unwrap().house)
        }
    }
    pub fn can_stack_fused(&self, ctx: &ReducerContext, unit: &FusedUnit) -> bool {
        unit.bases.len() == 1 && self.can_stack(ctx, &unit.bases[0])
    }
    pub fn can_fuse(a: &FusedUnit, b: &FusedUnit) -> bool {
        a.fusible() && b.fusible()
        // !a.bases.iter().any(|base| b.bases.contains(base))
    }
    pub fn fusible(&self) -> bool {
        self.lvl > (self.bases.len() as u32)
    }
    pub fn add_xp(&mut self, xp: u32) {
        self.xp += xp;
        while self.xp >= self.lvl {
            self.xp -= self.lvl;
            self.add_lvl();
        }
    }
    pub fn add_lvl(&mut self) {
        self.lvl += 1;
        self.pwr += 1;
        self.hp += 1;
    }
    pub fn total_xp(&self) -> u32 {
        self.lvl * (self.lvl - 1) / 2 + self.xp
    }
    pub fn add_fuse_xp(&mut self, source: &FusedUnit) {
        self.add_xp(source.total_xp() - 1);
    }
    pub fn rarity(&self, ctx: &ReducerContext) -> u8 {
        self.get_bases(ctx)
            .into_iter()
            .map(|u| u.rarity)
            .max()
            .unwrap_or_default()
    }
}
