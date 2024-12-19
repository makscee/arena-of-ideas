use std::fmt::Display;

use super::*;

pub struct BattlePlugin;

pub struct Battle {
    pub left: Vec<Unit>,
    pub right: Vec<Unit>,
}

#[derive(Clone, Copy, Debug)]
pub enum UnitId {
    UnitLeft(usize),
    UnitRight(usize),
}
#[derive(Clone, Copy, Debug)]
pub enum BattleAction {
    Strike(UnitId, UnitId),
    Damage(UnitId, UnitId, i32),
    Heal(UnitId, UnitId, i32),
    Death(UnitId),
}
impl BattleAction {
    fn apply(self, battle: &mut Battle) -> Self {
        match self {
            BattleAction::Strike(..) => {}
            BattleAction::Damage(_, b, x) => {
                battle.get_mut(b).stats.as_mut().unwrap().hp -= x;
            }
            BattleAction::Heal(_, b, x) => {
                battle.get_mut(b).stats.as_mut().unwrap().hp += x;
            }
            BattleAction::Death(a) => battle.remove(a),
        };
        self
    }
}

impl Battle {
    pub fn run(mut self) -> Vec<BattleAction> {
        let mut actions: Vec<BattleAction> = default();
        while !self.left.is_empty() && !self.right.is_empty() {
            for a in self.strike() {
                actions.push(a.apply(&mut self));
            }
            for a in self.death_check() {
                actions.push(a.apply(&mut self));
            }
        }
        actions
    }
    fn get_mut(&mut self, uid: UnitId) -> &mut Unit {
        match uid {
            UnitId::UnitLeft(i) => &mut self.left[i],
            UnitId::UnitRight(i) => &mut self.right[i],
        }
    }
    fn remove(&mut self, uid: UnitId) {
        match uid {
            UnitId::UnitLeft(i) => self.left.remove(i),
            UnitId::UnitRight(i) => self.right.remove(i),
        };
    }
    fn death_check(&mut self) -> Vec<BattleAction> {
        let mut actions: Vec<BattleAction> = default();
        for (i, u) in self.left.iter().enumerate() {
            if u.stats.as_ref().unwrap().hp <= 0 {
                actions.push(BattleAction::Death(UnitId::UnitLeft(i)));
            }
        }
        for (i, u) in self.right.iter().enumerate() {
            if u.stats.as_ref().unwrap().hp <= 0 {
                actions.push(BattleAction::Death(UnitId::UnitRight(i)));
            }
        }
        actions
    }
    fn strike(&mut self) -> Vec<BattleAction> {
        let mut actions: Vec<BattleAction> = default();
        let a = UnitId::UnitLeft(0);
        let b = UnitId::UnitRight(0);
        actions.push(BattleAction::Strike(a, b));
        actions.push(BattleAction::Damage(
            a,
            b,
            self.get_mut(a).stats.as_ref().unwrap().pwr,
        ));
        actions.push(BattleAction::Damage(
            b,
            a,
            self.get_mut(b).stats.as_ref().unwrap().pwr,
        ));

        actions
    }
}

impl ToCstr for UnitId {
    fn cstr(&self) -> Cstr {
        match self {
            UnitId::UnitLeft(i) => format!("l{i}"),
            UnitId::UnitRight(i) => format!("r{i}"),
        }
    }
}
impl Display for UnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cstr())
    }
}
impl ToCstr for BattleAction {
    fn cstr(&self) -> Cstr {
        match self {
            BattleAction::Strike(a, b) => format!("{a}|{b}"),
            BattleAction::Damage(a, b, x) => format!("{a}>{b}-{x}"),
            BattleAction::Heal(a, b, x) => format!("{a}>{b}+{x}"),
            BattleAction::Death(a) => format!("{a}x"),
        }
    }
}
