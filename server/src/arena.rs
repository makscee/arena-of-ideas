use std::mem;

use itertools::Itertools;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use spacetimedb::Timestamp;

use self::base_unit::TBaseUnit;

use super::*;

fn settings() -> GlobalSettings {
    GlobalSettings::get()
}

#[spacetimedb(table(public))]
pub struct TArenaRun {
    mode: GameMode,
    #[primarykey]
    id: u64,
    #[unique]
    owner: u64,
    team: u64,
    battles: Vec<u64>,
    shop_slots: Vec<ShopSlot>,
    team_slots: Vec<TeamSlot>,
    fusion: Option<Fusion>,
    g: i32,
    price_reroll: i32,
    free_rerolls: u32,
    lives: u32,
    active: bool,
    champion: Option<u64>,

    floor: u32,
    rerolls: u32,
    rewards: Vec<Reward>,
    streak: u32,

    last_updated: Timestamp,
}

#[spacetimedb(table(public))]
pub struct TArenaRunArchive {
    mode: GameMode,
    #[primarykey]
    id: u64,
    owner: u64,
    team: u64,
    battles: Vec<u64>,
    floor: u32,
    rewards: Vec<Reward>,
}

impl TArenaRunArchive {
    fn add_from_run(run: TArenaRun) {
        Self::insert(Self {
            mode: run.mode,
            id: run.id,
            owner: run.owner,
            team: run.team,
            battles: run.battles,
            floor: run.floor,
            rewards: run.rewards,
        })
        .expect("Failed to archive a run");
    }
}

#[derive(SpacetimeType, Clone, Default)]
pub struct ShopSlot {
    unit: String,
    id: u64,
    buy_price: i32,
    stack_price: i32,
    freeze: bool,
    discount: bool,
    available: bool,
    stack_targets: Vec<u8>,
    house_filter: Vec<String>,
}

#[derive(SpacetimeType, Clone, Default)]
pub struct TeamSlot {
    stack_targets: Vec<u8>,
    fuse_targets: Vec<u8>,
    sell_price: i32,
}

#[derive(SpacetimeType)]
pub struct Fusion {
    options: Vec<FusedUnit>,
    a: u8,
    b: u8,
}

#[derive(SpacetimeType)]
pub struct Reward {
    source: String,
    amount: i64,
}

#[spacetimedb(reducer)]
fn run_start_normal(ctx: ReducerContext) -> Result<(), String> {
    let user = ctx.user()?;
    TArenaRun::start(user, GameMode::ArenaNormal)
}

#[spacetimedb(reducer)]
fn run_start_ranked(ctx: ReducerContext, team_id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let cost = TPrices::filter_by_owner(&user.id).unwrap().buy_ranked();
    TWallet::change(user.id, -cost)?;
    let mut team = TTeam::get_owned(team_id, user.id)?;
    team.pool = TeamPool::Arena;
    let team = team.apply_limit().apply_empty_stat_bonus().save_clone();
    TArenaRun::start(user, GameMode::ArenaRanked)?;
    let mut run = TArenaRun::current(&ctx)?;
    run.team = team.id;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_start_const(ctx: ReducerContext) -> Result<(), String> {
    let user = ctx.user()?;
    let cost = TPrices::filter_by_owner(&user.id).unwrap().buy_const();
    TWallet::change(user.id, -cost)?;
    TArenaRun::start(
        user,
        GameMode::ArenaConst(GlobalData::get().constant_seed.clone()),
    )
}

#[spacetimedb(reducer)]
fn run_finish(ctx: ReducerContext) -> Result<(), String> {
    let run = TArenaRun::current(&ctx)?;
    let reward: i64 = run.rewards.iter().map(|r| r.amount).sum();
    TWallet::change(run.owner, reward)?;
    TArenaRun::delete_by_id(&run.id);
    TArenaRunArchive::add_from_run(run);
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_finish(ctx: ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let team = TTeam::get(run.team)?.apply_limit().save_clone();
    let (champion, enemy) = TArenaPool::get_next_enemy(&run.mode, run.floor);
    if champion {
        run.champion = Some(enemy);
    }
    run.battles
        .push(TBattle::new(run.mode.clone(), run.owner, team.id, enemy));

    if !team.units.is_empty() && !run.champion_reached() {
        TArenaPool::add(run.mode.clone(), team.id, run.floor);
    }
    run.rerolls = 0;
    run.fill_case()?;
    let ars = &settings().arena;
    run.g += ars.g_income.value(run.floor as i64) as i32;
    run.free_rerolls += ars.free_rerolls_income;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn submit_battle_result(ctx: ReducerContext, result: TBattleResult) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if !run.champion_reached() {
        run.floor += 1;
    }
    let bid = *run.battles.last().context_str("Last battle not present")?;
    let battle = TBattle::get(bid)?;
    if !battle.is_tbd() {
        return Err("Result already submitted".to_owned());
    }
    battle.set_result(result).save();
    if matches!(result, TBattleResult::Left) {
        run.add_streak();
        if run.champion_reached() {
            run.finish();
        }
    } else {
        run.finish_streak();
        run.lives -= 1;
        if run.lives == 0 {
            run.finish();
        }
    }
    let (champion, enemy) = TArenaPool::get_next_enemy(&run.mode, run.floor);
    if champion {
        run.champion = Some(enemy);
    }
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_reorder(ctx: ReducerContext, from: u8, to: u8) -> Result<(), String> {
    let run = TArenaRun::current(&ctx)?;
    let mut team = TTeam::get(run.team)?;
    let from = from as usize;
    let to = (to as usize).min(team.units.len() - 1);
    if team.units.len() < from {
        return Err("Wrong from index".into());
    }
    let unit = team.units.remove(from);
    team.units.insert(to, unit);
    team.save();
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_set_freeze(ctx: ReducerContext, slot: u8, value: bool) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let slot = run
        .shop_slots
        .get_mut(slot as usize)
        .context_str("Wrong slot index")?;
    if slot.freeze == value {
        if value {
            return Err("Slot already frozen".into());
        } else {
            return Err("Slot already unfrozen".into());
        }
    }
    slot.freeze = value;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_reroll(ctx: ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if run.free_rerolls > 0 {
        run.free_rerolls -= 1;
    } else {
        if run.g < run.price_reroll {
            return Err("Not enough G".into());
        }
        run.g -= run.price_reroll;
    }
    run.rerolls += 1;
    run.fill_case()?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_buy(ctx: ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let price = run.shop_slots[slot as usize].buy_price;
    let unit = run.buy(slot, price)?;
    run.add_to_team(FusedUnit::from_base_name(unit, next_id())?)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_sell(ctx: ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    run.sell(slot as usize)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_change_g(ctx: ReducerContext, delta: i32) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    run.g += delta;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_start(ctx: ReducerContext, a: u8, b: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let team = run.team()?;
    let fusion = Fusion {
        options: FusedUnit::fuse(team.get_unit(a)?, team.get_unit(b)?)?,
        a,
        b,
    };
    run.fusion = Some(fusion);
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_cancel(ctx: ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if run.fusion.is_none() {
        return Err("Fusion not started".to_owned());
    }
    run.fusion = None;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_choose(ctx: ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let Fusion { options, a, b } = run.fusion.take().context_str("Fusion not started")?;
    let slot = slot as usize;
    let a = a as usize;
    let b = b as usize;
    if slot >= options.len() {
        return Err("Wrong fusion index".to_owned());
    }
    let mut team = run.team()?;
    team.units.remove(a);
    team.units.insert(a, options[slot].clone());
    team.units.remove(b);
    team.save();
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn stack_shop(ctx: ReducerContext, source: u8, target: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let price = run.shop_slots[source as usize].stack_price;
    let unit = run.buy(source, price)?;
    let mut team = run.team()?;
    let target = team
        .units
        .get_mut(target as usize)
        .context_str("Team unit not found")?;
    if !target.can_stack(&unit) {
        return Err(format!(
            "Units {unit} and {} can not be stacked",
            target.bases.join("+")
        ));
    }
    target.add_xp(1);
    team.save();
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn stack_team(ctx: ReducerContext, source: u8, target: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let mut team = run.team()?;
    let source_unit = team.units[source as usize].clone();
    let target_unit = team
        .units
        .get_mut(target as usize)
        .context_str("Team unit not found")?;
    if !target_unit.can_stack_fused(&source_unit) {
        return Err(format!(
            "Units {} and {} can not be stacked",
            source_unit.bases.join("+"),
            target_unit.bases.join("+")
        ));
    }
    target_unit.add_xp(1);
    team.units.remove(source as usize);
    team.save();
    run.save();
    Ok(())
}

impl TArenaRun {
    fn new(user_id: u64, mode: GameMode) -> Self {
        let ars = settings().arena;
        Self {
            id: next_id(),
            owner: user_id,
            team: TTeam::new(user_id, TeamPool::Arena).save(),
            shop_slots: Vec::new(),
            team_slots: Vec::new(),
            fusion: None,
            floor: 0,
            rerolls: 0,
            last_updated: Timestamp::now(),
            g: ars.g_income.value(0) as i32,
            price_reroll: ars.price_reroll,
            battles: Vec::new(),
            lives: ars.lives_initial,
            free_rerolls: ars.free_rerolls_initial,
            active: true,
            champion: None,
            mode,
            rewards: Vec::new(),
            streak: 0,
        }
    }
    fn start(user: TUser, mode: GameMode) -> Result<(), String> {
        TArenaRun::delete_by_owner(&user.id);
        let mut run = TArenaRun::new(user.id, mode);
        run.fill_case()?;
        TArenaRun::insert(run)?;
        Ok(())
    }
    fn current(ctx: &ReducerContext) -> Result<Self, String> {
        Self::filter_by_owner(&ctx.user()?.id).context_str("No arena run in progress")
    }
    fn buy(&mut self, slot: u8, price: i32) -> Result<String, String> {
        let s = self
            .shop_slots
            .get_mut(slot as usize)
            .context_str("Wrong shop slot")?;
        if !s.available {
            return Err("Unit already bought".to_owned());
        }
        if price > self.g {
            return Err("Not enough G".to_owned());
        }
        self.g -= price;
        s.available = false;
        Ok(s.unit.clone())
    }
    fn add_to_team(&mut self, unit: FusedUnit) -> Result<(), String> {
        let mut team = self.team()?;
        team.units.insert(0, unit);
        team.save();
        Ok(())
    }
    fn sell(&mut self, slot: usize) -> Result<(), String> {
        self.remove_team(slot)?;
        self.g += self.team_slots[slot].sell_price;
        Ok(())
    }
    fn team(&mut self) -> Result<TTeam, String> {
        TTeam::get(self.team)
    }
    fn remove_team(&mut self, slot: usize) -> Result<FusedUnit, String> {
        let mut team = self.team()?;
        if team.units.len() > slot {
            let unit = team.units.remove(slot);
            team.save();
            return Ok(unit);
        } else {
            return Err("Slot is empty".to_owned());
        }
    }
    fn save(mut self) {
        self.last_updated = Timestamp::now();
        let team = TTeam::get(self.team).unwrap();
        for slot in &mut self.shop_slots {
            slot.stack_targets.clear();
            if !slot.available {
                continue;
            }
            for (i, unit) in team.units.iter().enumerate() {
                if unit.can_stack(&slot.unit) {
                    slot.stack_targets.push(i as u8);
                }
            }
        }
        let GlobalSettings {
            arena: ars,
            rarities: rs,
            ..
        } = &settings();
        self.team_slots = vec![TeamSlot::default(); team.units.len()];
        for (slot_i, slot) in self.team_slots.iter_mut().enumerate() {
            slot.stack_targets.clear();
            if let Some(unit) = team.units.get(slot_i) {
                let rarity = unit.rarity() as usize;
                slot.sell_price = rs.prices[rarity] - ars.sell_discount;
            }
            for (i, unit) in team.units.iter().enumerate() {
                if i == slot_i {
                    continue;
                }
                if unit.can_stack_fused(&team.units[slot_i]) {
                    slot.stack_targets.push(i as u8);
                }
                if FusedUnit::can_fuse(unit, &team.units[slot_i]) {
                    slot.fuse_targets.push(i as u8);
                }
            }
        }
        Self::update_by_owner(&self.owner.clone(), self);
    }
    fn fill_case(&mut self) -> Result<(), String> {
        let GlobalSettings {
            arena: ars,
            rarities,
            ..
        } = settings();
        let slots = ars.shop_slots.value(self.floor as i64) as usize;
        let mut old_slots = Vec::default();
        mem::swap(&mut self.shop_slots, &mut old_slots);
        for i in 0..slots {
            if let Some(slot) = old_slots.get(i) {
                if slot.available && slot.freeze {
                    self.shop_slots.push(slot.clone());
                    continue;
                }
            }
            self.shop_slots.push(ShopSlot::default());
        }
        let team = self.team()?;
        if !team.units.is_empty() {
            self.shop_slots[0].house_filter = team
                .units
                .iter()
                .flat_map(|u| u.get_houses())
                .unique()
                .collect();
        }
        let floor = self.floor as i32;
        let weights = rarities
            .weights_initial
            .iter()
            .enumerate()
            .map(|(i, w)| (*w + rarities.weights_per_floor[i] * floor).max(0))
            .collect_vec();
        let mut rng = self.get_rng();
        for i in 0..slots {
            let id = next_id();
            let s = &mut self.shop_slots[i];
            if s.freeze {
                continue;
            }
            s.available = true;
            s.id = id;
            let unit = TBaseUnit::get_random(&s.house_filter, &weights, &mut rng);
            s.unit = unit.name.clone();
            s.buy_price = rarities.prices[unit.rarity as usize];
            s.stack_price = s.buy_price - ars.stack_discount;
        }
        Ok(())
    }
    fn get_seed(&self) -> String {
        let Self {
            id, floor, rerolls, ..
        } = self;
        match &self.mode {
            GameMode::ArenaNormal | GameMode::ArenaRanked => format!("{id}_{floor}_{rerolls}"),
            GameMode::ArenaConst(seed) => format!("{seed}_{floor}_{rerolls}"),
        }
    }
    fn get_rng(&self) -> Pcg64 {
        Seeder::from(self.get_seed()).make_rng()
    }
    fn add_streak(&mut self) {
        self.streak += 1;
    }
    fn finish_streak(&mut self) {
        if self.streak > 0 {
            self.add_reward(
                format!("Streak {}", self.streak),
                (1..=self.streak as i64).sum(),
            );
        }
        self.streak = 0;
    }
    fn add_reward(&mut self, source: String, mut amount: i64) {
        amount *= match self.mode {
            GameMode::ArenaNormal => 1,
            GameMode::ArenaRanked => 2,
            GameMode::ArenaConst(_) => 3,
        };
        self.rewards.push(Reward { source, amount });
    }
    fn champion_reached(&self) -> bool {
        self.champion.is_some()
    }
    fn finish(&mut self) {
        self.active = false;
        self.finish_streak();
        if self.champion_reached() {
            if self
                .battles
                .last()
                .and_then(|id| TBattle::filter_by_id(id))
                .map(|b| b.result.is_win())
                .unwrap_or_default()
            {
                self.add_reward("Champion Defeated".into(), self.floor as i64 * 4 + 10);
                TArenaLeaderboard::insert(TArenaLeaderboard::new(
                    self.mode.clone(),
                    self.floor + 1,
                    self.owner,
                    self.team,
                    self.id,
                ));
            }
        }
    }
}
