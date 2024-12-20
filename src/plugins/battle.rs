use std::fmt::Display;

use ron::value;

use super::*;

pub struct BattlePlugin;

pub struct Battle {
    pub left: Vec<Unit>,
    pub right: Vec<Unit>,
}
pub struct BattleSimulation {
    pub units: HashMap<BID, Unit>,
    pub corpses: HashMap<BID, Unit>,
    pub left: Vec<BID>,
    pub right: Vec<BID>,
}
#[derive(Default, Debug)]
pub struct BattleLog {
    pub units: HashMap<BID, Unit>,
    pub actions: Vec<BattleAction>,
}
#[derive(Default)]
pub struct BattleHistory {
    pub t: f32,
    pub reps: HashMap<BID, Representation>,
    pub states: HashMap<BID, StateHistory>,
}
#[derive(Default)]
pub struct StateHistory {
    pub vars: HashMap<VarName, VarHistory>,
}
#[derive(Default)]
pub struct VarHistory {
    changes: Vec<(f32, VarValue)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BID {
    side: UnitSide,
    id: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitSide {
    Left,
    Right,
}
#[derive(Clone, Debug)]
pub enum BattleAction {
    VarSet(BID, BID, VarName, VarValue),
    Strike(BID, BID),
    Damage(BID, BID, i32),
    Death(BID),
}
impl BattleAction {
    fn apply_sim(&self, battle: &mut BattleSimulation) -> Vec<Self> {
        dbg!(self);
        match self {
            BattleAction::Strike(a, b) => {
                let pwr = battle.units.get(a).unwrap().stats.as_ref().unwrap().pwr;
                let action_a = Self::Damage(*a, *b, pwr);
                let pwr = battle.units.get(b).unwrap().stats.as_ref().unwrap().pwr;
                let action_b = Self::Damage(*b, *a, pwr);
                [action_a, action_b].into()
            }
            BattleAction::Death(a) => {
                battle.die(*a);
                default()
            }
            BattleAction::Damage(a, b, x) => {
                let hp = battle.get_mut(*b).stats.as_ref().unwrap().hp - x;
                [Self::VarSet(*a, *b, VarName::hp, hp.into())].into()
            }
            BattleAction::VarSet(_, b, var, value) => {
                battle.get_mut(*b).set_var(*var, value.clone());
                dbg!(battle.get_mut(*b).get_var(*var));
                default()
            }
        }
    }
    fn apply_hist(&self, history: &mut BattleHistory) {
        const ANIMATION: f32 = 0.1;
        match self {
            BattleAction::Strike(a, b) => {}
            BattleAction::VarSet(_, b, var, value) => {
                history.set_var(*b, *var, value.clone());
                history.t += ANIMATION;
            }
            BattleAction::Death(a) => {
                history.set_var(*a, VarName::visible, false.into());
            }
            BattleAction::Damage(bid, bid1, _) => todo!(),
        }
    }
}

impl VarHistory {
    fn new(value: VarValue) -> Self {
        Self {
            changes: vec![(0.0, value)],
        }
    }
}
impl StateHistory {
    fn from_vars(vars: Vec<(VarName, VarValue)>) -> Self {
        Self {
            vars: HashMap::from_iter(vars.into_iter().map(|(k, v)| (k, VarHistory::new(v)))),
        }
    }
    fn set(&mut self, t: f32, var: VarName, value: VarValue) {
        let h = &mut self.vars.entry(var).or_default().changes;
        h.push((t, value));
    }
}
impl BattleHistory {
    fn set_var(&mut self, id: BID, var: VarName, value: VarValue) {
        self.states.get_mut(&id).unwrap().set(self.t, var, value);
    }
}
impl BattleLog {
    pub fn generate_history(&self) -> BattleHistory {
        let mut bh = BattleHistory::default();
        for (id, u) in &self.units {
            bh.reps.insert(*id, u.representation.clone().unwrap());
            let vars = u.get_all_vars();
            bh.states.insert(*id, StateHistory::from_vars(vars));
        }
        for a in self.actions.iter() {
            a.apply_hist(&mut bh);
        }
        bh
    }
}
impl BattleSimulation {
    pub fn new(battle: &Battle) -> Self {
        let units = HashMap::from_iter(
            battle
                .left
                .iter()
                .enumerate()
                .map(|(id, u)| (BID::left(id), u.clone()))
                .chain(
                    battle
                        .right
                        .iter()
                        .enumerate()
                        .map(|(id, u)| (BID::right(id), u.clone())),
                ),
        );
        let left = (0..battle.left.len()).map(|i| BID::left(i)).collect_vec();
        let right = (0..battle.right.len()).map(|i| BID::right(i)).collect_vec();
        Self {
            units,
            left,
            right,
            corpses: default(),
        }
    }
    fn process_actions(&mut self, log: &mut BattleLog, mut actions: Vec<BattleAction>) {
        while let Some(a) = actions.pop() {
            log.actions.push(a.clone());
            actions.extend(a.apply_sim(self));
        }
    }
    pub fn run(mut self) -> BattleLog {
        let mut log = BattleLog::default();
        while !self.left.is_empty() && !self.right.is_empty() {
            let a = BattleAction::Strike(self.left[0], self.right[0]);
            self.process_actions(&mut log, [a].into());
            self.process_actions(&mut log, self.death_check());
        }
        log
    }
    fn get_mut(&mut self, id: BID) -> &mut Unit {
        self.units.get_mut(&id).unwrap()
    }
    fn death_check(&self) -> Vec<BattleAction> {
        let mut actions: Vec<BattleAction> = default();
        for (id, u) in self.units.iter() {
            if u.stats.as_ref().unwrap().hp <= 0 {
                actions.push(BattleAction::Death(*id));
            }
        }
        actions
    }
    fn die(&mut self, id: BID) {
        if let Some(u) = self.units.remove(&id) {
            self.corpses.insert(id, u);
        }
        if let Some(p) = self.left.iter().position(|u| *u == id) {
            self.left.remove(p);
        }
        if let Some(p) = self.right.iter().position(|u| *u == id) {
            self.right.remove(p);
        }
    }
}

impl ToCstr for UnitSide {
    fn cstr(&self) -> Cstr {
        match self {
            UnitSide::Left => "l".cstr_s(CstrStyle::Small),
            UnitSide::Right => "r".cstr_s(CstrStyle::Small),
        }
    }
}
impl ToCstr for BID {
    fn cstr(&self) -> Cstr {
        format!("{}{}", self.side.cstr(), self.id)
    }
}
impl Display for BID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cstr())
    }
}
impl BID {
    fn left(id: usize) -> Self {
        Self {
            side: UnitSide::Left,
            id,
        }
    }
    fn right(id: usize) -> Self {
        Self {
            side: UnitSide::Right,
            id,
        }
    }
}
impl ToCstr for BattleAction {
    fn cstr(&self) -> Cstr {
        match self {
            BattleAction::Strike(a, b) => format!("{a}|{b}"),
            BattleAction::Damage(a, b, x) => format!("{a}>{b}-{x}"),
            // BattleAction::Heal(a, b, x) => format!("{a}>{b}+{x}"),
            BattleAction::Death(a) => format!("{a}x"),
            BattleAction::VarSet(a, b, var, value) => format!("{a}>{b}${var}>{value}"),
        }
    }
}
