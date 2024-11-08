use std::mem;

use arena_leaderboard::arena_leaderboard;
use battle::battle;
use itertools::Itertools;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use spacetimedb::{table, Table, Timestamp};

use super::*;

#[table(public, name = arena_run)]
pub struct TArenaRun {
    mode: GameMode,
    #[primary_key]
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
    max_lives: u32,
    replenish_lives: u32,
    active: bool,
    boss_team: u64,
    boss_floor: u32,
    current_floor_boss: Option<u64>,

    floor: u32,
    rerolls: u32,
    rewards: Vec<Reward>,
    streak: u32,
    weights: Vec<i32>,

    last_updated: Timestamp,
}

#[table(public, name = arena_run_archive)]
pub struct TArenaRunArchive {
    #[primary_key]
    id: u64,
    season: u32,
    mode: GameMode,
    owner: u64,
    team: u64,
    battles: Vec<u64>,
    floor: u32,
    rewards: Vec<Reward>,
    ts: Timestamp,
}

impl TArenaRunArchive {
    fn add_from_run(ctx: &ReducerContext, run: TArenaRun) {
        ctx.db.arena_run_archive().insert(Self {
            id: run.id,
            mode: run.mode,
            season: GlobalSettings::get(ctx).season,
            owner: run.owner,
            team: run.team,
            battles: run.battles,
            floor: run.floor,
            rewards: run.rewards,
            ts: Timestamp::now(),
        });
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
    unit: FusedUnit,
    triggers: Vec<Vec<u32>>,
    targets: Vec<Vec<u32>>,
    effects: Vec<Vec<u32>>,
    a: u8,
    b: u8,
}

#[derive(SpacetimeType)]
pub struct Reward {
    source: String,
    amount: i64,
}

#[spacetimedb::reducer]
fn run_start_normal(ctx: &ReducerContext) -> Result<(), String> {
    let player = ctx.player()?;
    TArenaRun::start(ctx, player, GameMode::ArenaNormal)
}

#[spacetimedb::reducer]
fn run_start_ranked(ctx: &ReducerContext, team_id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let cost = TDailyState::get(ctx, player.id).buy_ranked(ctx);
    TWallet::change(ctx, player.id, -cost)?;
    let mut team = TTeam::get_owned(ctx, team_id, player.id)?;
    team.pool = TeamPool::Arena;
    let team = team
        .apply_limit(ctx)
        .apply_empty_stat_bonus(ctx)
        .save_clone(ctx);
    TArenaRun::start(ctx, player, GameMode::ArenaRanked)?;
    let mut run = TArenaRun::current(&ctx)?;
    run.team = team.id;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn run_start_const(ctx: &ReducerContext) -> Result<(), String> {
    let player = ctx.player()?;
    let cost = TDailyState::get(ctx, player.id).buy_const(ctx);
    TWallet::change(ctx, player.id, -cost)?;
    TArenaRun::start(ctx, player, GameMode::ArenaConst)
}

#[spacetimedb::reducer]
fn run_finish(ctx: &ReducerContext) -> Result<(), String> {
    let run = TArenaRun::current(&ctx)?;
    TPlayerGameStats::register_run_end(ctx, run.owner, run.mode, run.floor);
    let reward: i64 = run.rewards.iter().map(|r| r.amount).sum();
    TWallet::change(ctx, run.owner, reward)?;
    ctx.db.arena_run().id().delete(run.id);
    TArenaRunArchive::add_from_run(ctx, run);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_finish(ctx: &ReducerContext, face_boss: bool) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if face_boss {
        if let Some(boss) = TArenaLeaderboard::floor_boss(ctx, run.mode, run.floor) {
            run.boss_floor = boss.floor;
            run.boss_team = boss.team;
        }
    }

    let team = TTeam::get(ctx, run.team)?.apply_limit(ctx).save_clone(ctx);
    if run.boss_floor == run.floor {
        run.battles.push(TBattle::new(
            ctx,
            run.mode,
            run.owner,
            team.id,
            run.boss_team,
        ));
    } else {
        run.battles.push(TBattle::new(
            ctx,
            run.mode,
            run.owner,
            team.id,
            TArenaPool::get_next_enemy(ctx, &run.mode, run.floor),
        ));
    }
    if !team.units.is_empty() {
        TArenaPool::add(ctx, run.mode, team.id, run.floor);
    }

    run.rerolls = 0;
    run.fill_case(ctx)?;
    let ars = &GlobalSettings::get(ctx).arena;
    run.g += ars.g_income.value(run.floor as i64) as i32;
    run.free_rerolls += ars.free_rerolls_income;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn submit_battle_result(ctx: &ReducerContext, result: TBattleResult) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let bid = *run.battles.last().context_str("Last battle not present")?;
    let battle = TBattle::get(ctx, bid)?;
    if !battle.is_tbd() {
        return Err("Result already submitted".to_owned());
    }
    battle.set_result(ctx, result).save(ctx);
    let boss_floor = run.floor == run.boss_floor;
    if result == TBattleResult::Left {
        if !boss_floor
            || TArenaLeaderboard::current_champion(ctx, run.mode)
                .is_some_and(|a| a.floor == run.floor)
        {
            run.floor += 1;
        }
        if !boss_floor {
            run.update_boss(ctx);
        }
        QuestEvent::Win.register_event(ctx, run.mode, run.owner);
        if run.replenish_lives > 0 && run.lives < run.max_lives {
            run.lives += 1;
            run.replenish_lives -= 1;
        }
        run.add_streak(ctx);
        if boss_floor {
            QuestEvent::Champion.register_event(ctx, run.mode, run.owner);
            run.finish(ctx);
        }
    } else {
        run.finish_streak();
        run.lives -= 1;
        if run.lives == 0 {
            run.finish(ctx);
        }
    }
    if run.floor % 5 == 0 {
        run.replenish_lives += 1;
    }
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_reorder(ctx: &ReducerContext, from: u8, to: u8) -> Result<(), String> {
    let run = TArenaRun::current(&ctx)?;
    let mut team = TTeam::get(ctx, run.team)?;
    let from = from as usize;
    let to = (to as usize).min(team.units.len() - 1);
    if team.units.len() < from {
        return Err("Wrong from index".into());
    }
    let unit = team.units.remove(from);
    team.units.insert(to, unit);
    team.save(ctx);
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_set_freeze(ctx: &ReducerContext, slot: u8, value: bool) -> Result<(), String> {
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
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if run.mode == GameMode::ArenaNormal && run.floor == 0 && run.team(ctx)?.units.is_empty() {
    } else if run.free_rerolls > 0 {
        run.free_rerolls -= 1;
    } else {
        if run.g < run.price_reroll {
            return Err("Not enough G".into());
        }
        run.g -= run.price_reroll;
    }
    run.rerolls += 1;
    run.fill_case(ctx)?;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let price = run.shop_slots[slot as usize].buy_price;
    let unit = run.buy(ctx, slot, price)?;
    run.add_to_team(ctx, FusedUnit::from_base_name(ctx, unit, next_id(ctx))?)?;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_sell(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    run.sell(ctx, slot as usize)?;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn shop_change_g(ctx: &ReducerContext, delta: i32) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    run.g += delta;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn fuse_start(ctx: &ReducerContext, a: u8, b: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let team = run.team(ctx)?;
    let (unit, triggers, targets, effects) = FusedUnit::fuse(team.get_unit(a)?, team.get_unit(b)?)?;
    let fusion = Fusion {
        a,
        b,
        unit,
        triggers: triggers.into(),
        targets: targets.into(),
        effects: effects.into(),
    };
    run.fusion = Some(fusion);
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn fuse_cancel(ctx: &ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if run.fusion.is_none() {
        return Err("Fusion not started".to_owned());
    }
    run.fusion = None;
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn fuse_swap(ctx: &ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if let Some(fusion) = &mut run.fusion {
        fuse_start(ctx, fusion.b, fusion.a)
    } else {
        return Err("Fusion not started".to_owned());
    }
}

#[spacetimedb::reducer]
fn fuse_choose(ctx: &ReducerContext, trigger: i8, target: i8, effect: i8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if trigger + target + effect != 0 || trigger == target || target == effect {
        return Err("Choice condition failed".into());
    }
    let Fusion {
        mut unit,
        triggers,
        targets,
        effects,
        a,
        b,
    } = run.fusion.take().context_str("Fusion not started")?;
    unit.triggers = triggers
        .get((trigger + 1) as usize)
        .context_str("Failed to get trigger")?
        .clone();
    unit.targets = targets
        .get((target + 1) as usize)
        .context_str("Failed to get target")?
        .clone();
    unit.effects = effects
        .get((effect + 1) as usize)
        .context_str("Failed to get effect")?
        .clone();
    unit.id = next_id(ctx);

    let a = a as usize;
    let b = b as usize;
    let mut team = run.team(ctx)?;
    team.units.remove(a);
    team.units.insert(a, unit.clone());
    team.units.remove(b);
    team.save(ctx);
    let fuse_amount = unit.bases.len() as u32;
    QuestEvent::Fuse(fuse_amount).register_event(ctx, run.mode, run.owner);
    GlobalEvent::Fuse(unit).post(ctx, run.owner);
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn stack_shop(ctx: &ReducerContext, source: u8, target: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let price = run.shop_slots[source as usize].stack_price;
    let unit = run.buy(ctx, source, price)?;
    let mut team = run.team(ctx)?;
    let target = team
        .units
        .get_mut(target as usize)
        .context_str("Team unit not found")?;
    if !target.can_stack(ctx, &unit) {
        return Err(format!(
            "Units {unit} and {} can not be stacked",
            target.bases.join("+")
        ));
    }
    target.add_xp(1);
    team.save(ctx);
    run.save(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn stack_team(ctx: &ReducerContext, source: u8, target: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let mut team = run.team(ctx)?;
    let source_unit = team.units[source as usize].clone();
    let target_unit = team
        .units
        .get_mut(target as usize)
        .context_str("Team unit not found")?;
    if !target_unit.can_stack_fused(ctx, &source_unit) {
        return Err(format!(
            "Units {} and {} can not be stacked",
            source_unit.bases.join("+"),
            target_unit.bases.join("+")
        ));
    }
    target_unit.add_xp(1);
    team.units.remove(source as usize);
    team.save(ctx);
    run.save(ctx);
    Ok(())
}

impl TArenaRun {
    fn new(ctx: &ReducerContext, user_id: u64, mode: GameMode) -> Self {
        let ars = &GlobalSettings::get(ctx).arena;
        let mut c = Self {
            id: next_id(ctx),
            owner: user_id,
            team: TTeam::new(ctx, user_id, TeamPool::Arena).save(ctx),
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
            max_lives: ars.lives_initial,
            replenish_lives: 0,
            free_rerolls: ars.free_rerolls_initial,
            active: true,
            mode,
            rewards: Vec::new(),
            streak: 0,
            weights: default(),
            boss_team: 0,
            boss_floor: 0,
            current_floor_boss: None,
        };
        c.update_boss(ctx);
        c
    }
    fn start(ctx: &ReducerContext, player: TPlayer, mode: GameMode) -> Result<(), String> {
        ctx.db.arena_run().owner().delete(player.id);
        GlobalEvent::RunStart(mode).post(ctx, player.id);
        let mut run = TArenaRun::new(ctx, player.id, mode);
        run.fill_case(ctx)?;
        ctx.db.arena_run().insert(run);
        Ok(())
    }
    fn current(ctx: &ReducerContext) -> Result<Self, String> {
        ctx.db
            .arena_run()
            .owner()
            .find(ctx.player()?.id)
            .context_str("No arena run in progress")
    }
    fn buy(&mut self, ctx: &ReducerContext, slot: u8, price: i32) -> Result<String, String> {
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
        GlobalEvent::GameShopBuy(s.unit.clone()).post(ctx, self.owner);
        self.g -= price;
        s.available = false;
        Ok(s.unit.clone())
    }
    fn add_to_team(&mut self, ctx: &ReducerContext, unit: FusedUnit) -> Result<(), String> {
        let mut team = self.team(ctx)?;
        team.units.insert(0, unit);
        team.save(ctx);
        Ok(())
    }
    fn sell(&mut self, ctx: &ReducerContext, slot: usize) -> Result<(), String> {
        GlobalEvent::GameShopSell(self.team(ctx)?.units[slot].clone()).post(ctx, self.owner);
        self.remove_team(ctx, slot)?;
        self.g += self.team_slots[slot].sell_price;
        Ok(())
    }
    fn team(&mut self, ctx: &ReducerContext) -> Result<TTeam, String> {
        TTeam::get(ctx, self.team)
    }
    fn remove_team(&mut self, ctx: &ReducerContext, slot: usize) -> Result<FusedUnit, String> {
        let mut team = self.team(ctx)?;
        if team.units.len() > slot {
            let unit = team.units.remove(slot);
            team.save(ctx);
            return Ok(unit);
        } else {
            return Err("Slot is empty".to_owned());
        }
    }
    fn save(mut self, ctx: &ReducerContext) {
        self.last_updated = Timestamp::now();
        let team = TTeam::get(ctx, self.team).unwrap();
        for slot in &mut self.shop_slots {
            slot.stack_targets.clear();
            if !slot.available {
                continue;
            }
            for (i, unit) in team.units.iter().enumerate() {
                if unit.can_stack(ctx, &slot.unit) {
                    slot.stack_targets.push(i as u8);
                }
            }
        }
        let GlobalSettings {
            arena: ars,
            rarities: rs,
            ..
        } = &GlobalSettings::get(ctx);
        self.team_slots = vec![TeamSlot::default(); team.units.len()];
        for (slot_i, slot) in self.team_slots.iter_mut().enumerate() {
            slot.stack_targets.clear();
            if let Some(unit) = team.units.get(slot_i) {
                let rarity = unit.rarity(ctx) as usize;
                slot.sell_price = rs.prices[rarity] - ars.sell_discount;
            }
            for (i, unit) in team.units.iter().enumerate() {
                if i == slot_i {
                    continue;
                }
                if unit.can_stack_fused(ctx, &team.units[slot_i]) {
                    slot.stack_targets.push(i as u8);
                }
                if FusedUnit::can_fuse(unit, &team.units[slot_i]) {
                    slot.fuse_targets.push(i as u8);
                }
            }
        }
        ctx.db.arena_run().id().update(self);
    }
    fn fill_case(&mut self, ctx: &ReducerContext) -> Result<(), String> {
        let GlobalSettings {
            arena: ars,
            rarities,
            ..
        } = GlobalSettings::get(ctx);
        let slots = ars.shop_slots.value(self.floor as i64) as usize;
        let mut old_slots = Vec::default();
        mem::swap(&mut self.shop_slots, &mut old_slots);
        for i in 0..slots {
            if let Some(slot) = old_slots.get(i) {
                if slot.available {
                    if slot.freeze {
                        self.shop_slots.push(slot.clone());
                        continue;
                    } else {
                        GlobalEvent::GameShopSkip(slot.unit.clone()).post(ctx, self.owner);
                    }
                }
            }
            self.shop_slots.push(ShopSlot::default());
        }
        let team = self.team(ctx)?;
        if !team.units.is_empty() {
            self.shop_slots[0].house_filter = team
                .units
                .iter()
                .flat_map(|u| u.get_houses(ctx))
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
        self.weights = weights.clone();
        let mut rng = self.get_rng(ctx);
        for i in 0..slots {
            let id = next_id(ctx);
            let s = &mut self.shop_slots[i];
            if s.freeze {
                continue;
            }
            s.available = true;
            s.id = id;
            let unit = TBaseUnit::get_random(ctx, &s.house_filter, &weights, &mut rng);
            s.unit = unit.name.clone();
            s.buy_price = rarities.prices[unit.rarity as usize];
            s.stack_price = s.buy_price - ars.stack_discount;
        }
        Ok(())
    }
    fn get_seed(&self, ctx: &ReducerContext) -> String {
        let Self {
            id, floor, rerolls, ..
        } = self;
        match &self.mode {
            GameMode::ArenaNormal | GameMode::ArenaRanked => format!("{id}_{floor}_{rerolls}"),
            GameMode::ArenaConst => {
                format!("{}_{floor}_{rerolls}", GlobalSettings::get(ctx).season)
            }
        }
    }
    fn get_rng(&self, ctx: &ReducerContext) -> Pcg64 {
        Seeder::from(self.get_seed(ctx)).make_rng()
    }
    fn add_streak(&mut self, ctx: &ReducerContext) {
        self.streak += 1;
        QuestEvent::Streak(self.streak).register_event(ctx, self.mode, self.owner);
    }
    fn finish_streak(&mut self) {
        if self.streak > 0 {
            self.add_reward(
                format!("Streak {}", self.streak),
                (1..=self.streak as i64).sum(),
                0,
            );
        }
        self.streak = 0;
    }
    fn add_reward(&mut self, source: String, mut amount: i64, min_mul: i64) {
        amount *= match self.mode {
            GameMode::ArenaNormal => 0,
            GameMode::ArenaRanked => 1,
            GameMode::ArenaConst => 2,
        }
        .max(min_mul);
        self.rewards.push(Reward { source, amount });
    }
    fn update_boss(&mut self, ctx: &ReducerContext) {
        (self.boss_floor, self.boss_team) = TArenaLeaderboard::current_champion(ctx, self.mode)
            .map(|c| (c.floor, c.team))
            .unwrap_or_else(|| {
                (
                    GlobalSettings::get(ctx).arena.initial_enemies_count[self.mode as usize] - 1,
                    *GlobalData::get(ctx).initial_enemies.last().unwrap(),
                )
            });
        self.current_floor_boss =
            TArenaLeaderboard::floor_boss(ctx, self.mode, self.floor).map(|a| a.team)
    }
    fn finish(&mut self, ctx: &ReducerContext) {
        self.active = false;
        self.finish_streak();
        if self.floor >= self.boss_floor {
            if let Some(battle) = self
                .battles
                .last()
                .and_then(|id| ctx.db.battle().id().find(*id))
            {
                if battle.result.is_win() {
                    if TArenaLeaderboard::current_champion(ctx, self.mode)
                        .map(|c| c.floor <= self.floor)
                        .unwrap_or(true)
                    {
                        let reward = match self.mode {
                            GameMode::ArenaNormal => 0,
                            GameMode::ArenaRanked | GameMode::ArenaConst => 25,
                        };
                        self.add_reward(
                            "Champion Defeated".into(),
                            reward + self.floor as i64 * 4 + 10,
                            1,
                        );
                        TPlayerGameStats::register_champion(ctx, self.owner, self.mode);
                    } else {
                        let reward = match self.mode {
                            GameMode::ArenaNormal => 10,
                            GameMode::ArenaRanked | GameMode::ArenaConst => 10,
                        };
                        self.add_reward(
                            "Boss Defeated".into(),
                            reward + self.floor as i64 * 2 + 10,
                            1,
                        );
                        TPlayerGameStats::register_boss(ctx, self.owner, self.mode);
                    }
                    ctx.db.arena_leaderboard().insert(TArenaLeaderboard::new(
                        ctx,
                        self.mode,
                        self.floor,
                        self.owner,
                        battle.team_left,
                        self.id,
                    ));
                }
            }
        }
    }
}
