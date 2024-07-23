use itertools::Itertools;
use spacetimedb::Timestamp;

use self::base_unit::TBaseUnit;

use super::*;

#[derive(SpacetimeType)]
pub struct ArenaSettings {
    slots_min: u32,
    slots_max: u32,
    slots_per_round: f32,
    g_start: i32,
    g_income_min: i32,
    g_income_max: i32,
    g_income_per_round: i32,
    price_reroll: i32,
    sell_discount: i32,
    stack_discount: i32,
    team_slots: u32,
}

#[derive(SpacetimeType)]
pub struct RaritySettings {
    pub prices: Vec<i32>,
    pub weights_initial: Vec<i32>,
    pub weights_per_round: Vec<i32>,
}

fn settings() -> GlobalSettings {
    GlobalSettings::get()
}

#[spacetimedb(table)]
struct TArenaRun {
    #[primarykey]
    id: GID,
    #[unique]
    owner: GID,
    team: GID,
    battles: Vec<GID>,
    shop_slots: Vec<ShopSlot>,
    team_slots: Vec<TeamSlot>,
    fusion: Option<Fusion>,
    g: i32,
    price_reroll: i32,
    lives: u32,
    active: bool,

    round: u32,
    score: u32,

    last_updated: Timestamp,
}

#[spacetimedb(table)]
struct TArenaRunArchive {
    #[primarykey]
    id: GID,
    owner: GID,
    team: GID,
    battles: Vec<GID>,
    round: u32,
}

impl TArenaRunArchive {
    fn add_from_run(run: &TArenaRun) {
        Self::insert(Self {
            id: run.id,
            owner: run.owner,
            team: run.team,
            battles: run.battles.clone(),
            round: run.round,
        })
        .expect("Failed to archive a run");
    }
}

#[derive(SpacetimeType, Clone, Default)]
struct ShopSlot {
    unit: String,
    id: GID,
    buy_price: i32,
    stack_price: i32,
    freeze: bool,
    discount: bool,
    available: bool,
    stack_targets: Vec<u8>,
    house_filter: Vec<String>,
}

#[derive(SpacetimeType, Clone, Default)]
struct TeamSlot {
    stack_targets: Vec<u8>,
    fuse_targets: Vec<u8>,
    sell_price: i32,
}

#[derive(SpacetimeType)]
struct Fusion {
    options: Vec<FusedUnit>,
    source: u8,
    target: u8,
}

#[spacetimedb(reducer)]
fn run_start(ctx: ReducerContext) -> Result<(), String> {
    let user = TUser::find_by_identity(&ctx.sender)?;
    TArenaRun::delete_by_owner(&user.id);
    let mut run = TArenaRun::new(user.id);
    run.fill_case()?;
    TArenaRun::insert(run)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn run_finish(ctx: ReducerContext) -> Result<(), String> {
    let run = TArenaRun::current(&ctx)?;
    if TArenaLeaderboard::filter_by_round(&run.round).count() == 0 {
        TArenaLeaderboard::insert(TArenaLeaderboard::new(
            run.round, run.score, run.owner, run.team, run.id,
        ));
    }
    TArenaRun::delete_by_id(&run.id);
    TArenaRunArchive::add_from_run(&run);
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_finish(ctx: ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    run.round += 1;
    let team = TTeam::get(run.team)?.save_clone();
    let champion = TArenaLeaderboard::iter()
        .max_by_key(|d| d.round)
        .unwrap_or_default();
    let enemy = if run.round == champion.round {
        champion.team
    } else {
        TArenaPool::get_random(run.round)
            .map(|t| t.team)
            .unwrap_or_default()
    };
    run.battles.push(TBattle::new(run.owner, team.id, enemy));
    if !team.units.is_empty() {
        TArenaPool::add(team.id, run.round);
    }
    run.fill_case()?;
    run.g += 4;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn submit_battle_result(ctx: ReducerContext, result: TBattleResult) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let bid = *run.battles.last().context_str("Last battle not present")?;
    let mut battle = TBattle::get(bid)?;
    let is_no_enemy = battle.team_right == 0;
    if !matches!(battle.result, TBattleResult::Tbd) {
        return Err("Result already submitted".to_owned());
    }
    battle.result = result;
    battle.save();
    if matches!(result, TBattleResult::Right) {
        run.lives -= 1;
    } else {
        run.score += run.round;
    }
    if run.lives == 0 || is_no_enemy {
        run.active = false;
    }
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_reorder(ctx: ReducerContext, from: u8, to: u8) -> Result<(), String> {
    let run = TArenaRun::current(&ctx)?;
    let mut team = TTeam::get(run.team)?;
    let from = from as usize;
    let to = to as usize;
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
fn shop_reroll(ctx: ReducerContext) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    if run.g < run.price_reroll {
        return Err("Not enough G".into());
    }
    run.g -= run.price_reroll;
    run.fill_case()?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_buy(ctx: ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let price = run.shop_slots[slot as usize].buy_price;
    let unit = run.buy(slot, price)?;
    run.add_to_team(FusedUnit::from_base(unit, next_id()))?;
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
fn fuse_start(ctx: ReducerContext, target: u8, source: u8) -> Result<(), String> {
    let mut run = TArenaRun::current(&ctx)?;
    let team = run.team()?;

    let mut fusion = Fusion {
        options: Vec::default(),
        source,
        target,
    };
    let options = &mut fusion.options;
    let source = team.get_unit(source)?;
    let target = team.get_unit(target)?;
    if !target.can_fuse_source(source) {
        return Err(format!(
            "Can't fuse {} with {}",
            source.name(),
            target.name()
        ));
    }
    if source.bases.len() != 1 {
        return Err("Source can only be non-fused unit".to_owned());
    }
    let mut option = target.clone();
    option.bases.push(source.bases[0].clone());
    option.add_fuse_xp(source);
    let i = option.bases.len() as u32 - 1;
    if !source.triggers.is_empty() {
        let mut option = option.clone().new_id();
        option.triggers.push(i);
        options.push(option);
    }
    if !source.targets.is_empty() {
        let mut option = option.clone().new_id();
        option.targets.push(i);
        options.push(option);
    }
    if !source.effects.is_empty() {
        let mut option = option.clone().new_id();
        option.effects.push(i);
        options.push(option);
    }
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
    let Fusion {
        options,
        source,
        target,
    } = run.fusion.take().context_str("Fusion not started")?;
    let slot = slot as usize;
    let target = target as usize;
    let source = source as usize;
    if slot >= options.len() {
        return Err("Wrong fusion index".to_owned());
    }
    let mut team = run.team()?;
    team.units.remove(target);
    team.units.insert(target, options[slot].clone());
    team.units.remove(source);
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
    fn new(user_id: GID) -> Self {
        let ars = settings().arena;
        Self {
            id: next_id(),
            owner: user_id,
            team: TTeam::new(user_id),
            shop_slots: Vec::new(),
            team_slots: Vec::new(),
            fusion: None,
            round: 0,
            score: 0,
            last_updated: Timestamp::now(),
            g: ars.g_start,
            price_reroll: ars.price_reroll,
            battles: Vec::new(),
            lives: 1,
            active: true,
        }
    }
    fn current(ctx: &ReducerContext) -> Result<Self, String> {
        Self::filter_by_owner(&TUser::find_by_identity(&ctx.sender)?.id)
            .context_str("No arena run in progress")
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
                if unit.can_fuse_source(&team.units[slot_i]) {
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
        let slots = (ars.slots_min + (ars.slots_per_round * self.round as f32) as u32)
            .min(ars.slots_max) as usize;
        self.shop_slots = vec![ShopSlot::default(); slots];
        let team = self.team()?;
        if !team.units.is_empty() {
            self.shop_slots[0].house_filter = team
                .units
                .iter()
                .flat_map(|u| u.get_houses())
                .unique()
                .collect();
        }
        let round = self.round as i32;
        let weights = rarities
            .weights_initial
            .iter()
            .enumerate()
            .map(|(i, w)| (*w + rarities.weights_per_round[i] * round).max(0))
            .collect_vec();
        for i in 0..slots {
            let id = next_id();
            let s = &mut self.shop_slots[i];
            s.available = true;
            s.id = id;
            let unit = TBaseUnit::get_random(&s.house_filter, &weights);
            s.unit = unit.name.clone();
            s.buy_price = rarities.prices[unit.rarity as usize];
            s.stack_price = s.buy_price - ars.stack_discount;
        }
        Ok(())
    }
}