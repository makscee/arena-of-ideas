use rand::{seq::IteratorRandom, thread_rng};
use spacetimedb::Timestamp;

use self::base_unit::BaseUnit;

use super::*;

#[spacetimedb(table)]
struct Run {
    #[primarykey]
    id: GID,
    #[unique]
    owner: GID,
    team: GID,
    battles: Vec<GID>,
    shop: Vec<ShopSlot>,
    fusion: Option<Fusion>,
    g: i32,
    price_reroll: i32,
    price_unit: i32,
    price_sell: i32,
    lives: u32,
    active: bool,

    round: u32,

    last_updated: Timestamp,
}

#[spacetimedb(table)]
struct RunArchive {
    #[primarykey]
    id: GID,
    owner: GID,
    team: GID,
    battles: Vec<GID>,
    round: u32,
}

impl RunArchive {
    fn add_from_run(run: &Run) {
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
    id: u64,
    price: i32,
    freeze: bool,
    discount: bool,
    available: bool,
}

#[derive(SpacetimeType)]
struct Fusion {
    options: Vec<FusedUnit>,
    source: u8,
    target: u8,
}

#[spacetimedb(reducer)]
fn run_start(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    Run::delete_by_owner(&user.id);
    let mut run = Run::new(user.id);
    run.fill_case();
    Run::insert(run)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn run_finish(ctx: ReducerContext) -> Result<(), String> {
    let run = Run::current(&ctx)?;
    Run::delete_by_id(&run.id);
    RunArchive::add_from_run(&run);
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_finish(ctx: ReducerContext) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    let team = TTeam::get(run.team)?.save_clone();
    let enemy = TArenaPool::get_random(run.round)
        .map(|t| t.team)
        .unwrap_or_default();
    run.battles.push(TBattle::new(run.owner, team.id, enemy));
    if !team.units.is_empty() {
        TArenaPool::add(team.id, run.round);
    }
    run.round += 1;
    run.fill_case();
    run.g += 4;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn submit_battle_result(ctx: ReducerContext, result: TBattleResult) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
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
    }
    if run.lives == 0 || is_no_enemy {
        run.active = false;
    }
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_reroll(ctx: ReducerContext) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    if run.g < run.price_reroll {
        return Err("Not enough G".into());
    }
    run.g -= run.price_reroll;
    run.fill_case();
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_buy(ctx: ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    run.buy(slot)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_sell(ctx: ReducerContext, slot: u8) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    run.sell(slot as usize)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn shop_change_g(ctx: ReducerContext, delta: i32) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    run.g += delta;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_start(ctx: ReducerContext, target: u8, source: u8) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    // let source_unit = run.get_team_mut(source)?.clone();
    // if source_unit.bases.len() != 1 {
    //     return Err("Source can only be non-fused unit".to_owned());
    // }
    // let target_unit = run.get_team_mut(target)?.clone();
    // run.fusion = Some(Fusion {
    //     options: Vec::default(),
    //     source,
    //     target,
    // });
    // let options = &mut run.fusion.as_mut().unwrap().options;
    // TTeam::filter_by_id(&run.team);

    // let mut target_trigger = target_unit.clone();
    // target_trigger
    //     .triggers
    //     .extend(source_unit.triggers.clone().into_iter());
    // options.push(target_trigger);

    // let mut target_target = target_unit.clone();
    // target_target
    //     .targets
    //     .extend(source_unit.targets.clone().into_iter());
    // options.push(target_target);

    // let mut target_effect = target_unit.clone();
    // target_effect
    //     .effects
    //     .extend(source_unit.effects.clone().into_iter());
    // options.push(target_effect);
    // run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_cancel(ctx: ReducerContext) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    if run.fusion.is_none() {
        return Err("Fusion not started".to_owned());
    }
    run.fusion = None;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_choose(ctx: ReducerContext, ind: u8) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    // let fusion = run.fusion.take().context_str("Fusion not started")?;
    // if fusion.options.len() > ind as usize {
    //     return Err("Wrong fusion index".to_owned());
    // }
    // run.remove_team(fusion.source)?;
    // run.remove_team(fusion.target)?;
    // let slot = fusion.source.min(fusion.target);
    // let mut team = TTeam::get(run.team)?;
    // *team
    //     .units
    //     .get_mut(slot as usize)
    //     .context_str("Fusion insert error")? = fusion
    //     .options
    //     .get(ind as usize)
    //     .cloned()
    //     .context_str("Fusion option get error")?;
    // team.save();
    // run.fusion = None;
    // run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn stack(ctx: ReducerContext, source: u8, target: u8) -> Result<(), String> {
    let mut run = Run::current(&ctx)?;
    // let source = run.remove_team(source)?;
    // let target = run.get_team_mut(target)?;
    // if source.bases.len() != 1 {
    //     return Err("Fused unit can't be stack source".to_owned());
    // }
    // if !(target.stacks == 1
    //     && source
    //         .bases
    //         .first()
    //         .unwrap()
    //         .eq(target.bases.first().unwrap()))
    // {
    //     return Err("First level unit can only be stacked with same unit".to_owned());
    // }
    // if !target
    //     .get_houses()
    //     .contains(&source.get_houses().first().unwrap())
    // {
    //     return Err("Source house has to be one of target houses".to_owned());
    // }
    // target.stacks += source.stacks;
    // run.save();
    Ok(())
}

impl Run {
    fn new(user_id: u64) -> Self {
        let gs = GlobalSettings::get();
        Self {
            id: next_id(),
            owner: user_id,
            team: TTeam::new(user_id),
            shop: vec![ShopSlot::default(); gs.shop_slots_max as usize],
            fusion: None,
            round: 0,
            last_updated: Timestamp::now(),
            g: gs.shop_g_start,
            price_reroll: gs.shop_price_reroll,
            price_unit: gs.shop_price_unit,
            price_sell: gs.shop_price_sell,
            battles: Vec::new(),
            lives: 1,
            active: true,
        }
    }
    fn current(ctx: &ReducerContext) -> Result<Self, String> {
        Self::filter_by_owner(&User::find_by_identity(&ctx.sender)?.id)
            .context_str("No arena run in progress")
    }
    fn buy(&mut self, slot: u8) -> Result<(), String> {
        let s = self
            .shop
            .get_mut(slot as usize)
            .context_str("Wrong shop slot")?;
        if !s.available {
            return Err("Unit already bought".to_owned());
        }
        if s.price > self.g {
            return Err("Not enough G".to_owned());
        }
        self.g -= s.price;
        s.available = false;
        let unit = FusedUnit::from_base(s.unit.clone(), next_id());
        let mut team = self.team()?;
        team.units.insert(0, unit);
        team.save();
        Ok(())
    }
    fn sell(&mut self, slot: usize) -> Result<(), String> {
        self.remove_team(slot)?;
        self.g += self.price_sell;
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
        Self::update_by_owner(&self.owner.clone(), self);
    }
    fn fill_case(&mut self) {
        let gs = GlobalSettings::get();
        let slots = (gs.shop_slots_min + (gs.shop_slots_per_round * self.round as f32) as u32)
            .min(gs.shop_slots_max) as usize;
        self.shop = vec![ShopSlot::default(); slots];
        for i in 0..slots {
            let id = next_id();
            let s = &mut self.shop[i];
            s.available = true;
            s.price = self.price_unit;
            s.id = id;
            s.unit = BaseUnit::iter().choose(&mut thread_rng()).unwrap().name;
        }
    }
}
