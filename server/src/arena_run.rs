use std::collections::HashSet;

use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng};
use spacetimedb::Timestamp;

use crate::unit::TableUnit;

use super::*;

#[spacetimedb(table)]
pub struct ArenaRun {
    #[primarykey]
    #[autoinc]
    id: u64,
    user_id: u64,
    battles: Vec<ArenaBattle>,
    round: u32,
    state: RunState,
    active: bool,
    last_updated: Timestamp,
}
#[derive(SpacetimeType)]
pub struct ArenaBattle {
    enemy: u64,
    result: Option<bool>,
}
#[derive(SpacetimeType)]
pub struct RunState {
    g: i64,
    free_rerolls: u32,
    team: Vec<TeamUnit>,
    case: Vec<ShopOffer>,
    next_id: u64,
}
#[derive(SpacetimeType, Clone)]
pub struct TeamUnit {
    id: u64,
    unit: TableUnit,
}
#[derive(SpacetimeType, Clone)]
pub struct ShopOffer {
    available: bool,
    discount: bool,
    unit: TeamUnit,
}

#[spacetimedb(reducer)]
fn run_start(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    if let Some(mut run) = ArenaRun::filter_by_user_id(&user.id).find(|r| r.active) {
        run.active = false;
        run.save();
    }
    let mut run = ArenaRun::new(user.id);
    if let Some(enemy) = ArenaPool::filter_by_round(&0).choose(&mut thread_rng()) {
        run.battles.push(ArenaBattle {
            enemy: enemy.id,
            result: None,
        });
    }
    run.fill_case();
    ArenaRun::insert(run).unwrap();
    Ok(())
}
#[spacetimedb(reducer)]
fn run_submit_result(ctx: ReducerContext, win: bool) -> Result<(), String> {
    let (user_id, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let team = run
        .state
        .team
        .clone()
        .into_iter()
        .map(|u| u.unit)
        .collect_vec();
    ArenaPool::insert(ArenaPool {
        id: 0,
        owner: user_id,
        round: run.round,
        team,
    })?;
    if let Some(last_battle) = run.battles.get_mut(run.round as usize) {
        last_battle.result = Some(win);
    } else {
        run.active = false;
    }
    run.round += 1;

    if run.loses() > 2 {
        run.active = false;
    } else {
        if let Some(enemy) = ArenaBattle::next(&run) {
            run.battles.push(enemy);
        }
    }
    if run.active {
        let settings = GlobalSettings::get();
        run.change_g((settings.g_per_round_min + run.round as i64).min(settings.g_per_round_max));
        run.state.case.clear();
        run.fill_case();
        run.state.free_rerolls = 1;
    }
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_reroll(ctx: ReducerContext, force: bool) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let reroll_price = GlobalSettings::get().price_reroll;
    let mut pay = !force;
    if pay && run.state.free_rerolls > 0 {
        pay = false;
        run.state.free_rerolls -= 1;
    }
    if !pay || run.can_afford(reroll_price) {
        if pay {
            run.change_g(-reroll_price);
        }
        run.state.case.clear();
        run.fill_case();
        run.save();
        Ok(())
    } else {
        Err("Not enough g".to_owned())
    }
}

#[spacetimedb(reducer)]
fn run_buy(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let gs = GlobalSettings::get();
    let price = if run.find_offer(id)?.1.discount {
        gs.price_unit_sell
    } else {
        gs.price_unit_buy
    };
    run.buy(id, 0, price, false)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_sell(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let index = run
        .state
        .team
        .iter()
        .position(|u| u.id.eq(&id))
        .context_str("Unit not found")?;
    run.state.team.remove(index);
    run.change_g(GlobalSettings::get().price_unit_sell);
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_stack(ctx: ReducerContext, target: u64, source: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let i_source = if let Ok((ind, _)) = run.find_team(source) {
        ind
    } else {
        let gs = GlobalSettings::get();
        let price = if run.find_offer(source)?.1.discount {
            gs.price_unit_sell
        } else {
            gs.price_unit_buy_stack
        };
        if !run.can_afford(price) {
            return Err("Can't afford".to_owned());
        }
        run.buy(source, 0, price, true)?;
        run.find_team(source)?.0
    };
    let (i_target, _) = run.find_team(target)?;
    let target = &mut run.state.team[i_target].unit;
    target.pwr += 1;
    target.hp += 1;
    target.stacks += 1;
    if target.stacks >= target.level + 1 {
        target.stacks -= target.level;
        target.level += 1;
    }
    run.state.team.remove(i_source);
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_team_reorder(ctx: ReducerContext, order: Vec<u64>) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    run.state
        .team
        .sort_by_key(|u| order.iter().position(|o| u.id.eq(o)));
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_change_g(ctx: ReducerContext, delta: i64) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    run.change_g(delta);
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_fuse(ctx: ReducerContext, a: u64, b: u64, unit: TableUnit) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    run.state
        .team
        .retain(|TeamUnit { id, .. }| !b.eq(id) && !a.eq(id));
    let unit = TeamUnit {
        id: run.next_id(),
        unit,
    };
    run.state.team.insert(0, unit);
    run.save();
    Ok(())
}

impl ArenaRun {
    fn new(user_id: u64) -> Self {
        Self {
            user_id,
            active: true,
            last_updated: Timestamp::now(),
            id: 0,
            battles: Vec::default(),
            round: 0,
            state: RunState {
                g: GlobalSettings::get().g_per_round_min,
                team: Vec::default(),
                case: Vec::default(),
                next_id: 0,
                free_rerolls: 2,
            },
        }
    }

    fn loses(&self) -> u32 {
        self.battles
            .iter()
            .filter(|b| b.result.is_some_and(|r| !r))
            .count() as u32
    }

    fn get_by_identity(identity: &Identity) -> Result<(u64, Self), String> {
        let user_id = User::find_by_identity(identity)?.id;
        Ok((user_id, Self::get_active(&user_id)?))
    }

    fn get_active(user_id: &u64) -> Result<Self, String> {
        ArenaRun::filter_by_user_id(user_id)
            .find(|r| r.active)
            .context_str("No arena run in progress")
    }

    fn can_afford(&self, price: i64) -> bool {
        self.state.g >= price
    }
    fn change_g(&mut self, delta: i64) {
        self.state.g += delta;
    }

    fn next_id(&mut self) -> u64 {
        self.state.next_id += 1;
        self.state.next_id
    }

    fn fill_case(&mut self) {
        let settings = GlobalSettings::get();
        let slots = (settings.shop_slots_min
            + (self.round as f32 * settings.shop_slots_per_round) as u32)
            .min(settings.shop_slots_max);
        let team_houses: HashSet<String> = HashSet::from_iter(
            self.state
                .team
                .iter()
                .map(|u| u.unit.houses.split("+"))
                .flatten()
                .map(|s| s.to_owned())
                .collect_vec(),
        );
        for i in 0..slots {
            let id = self.next_id();
            self.state.case.push(
                TableUnit::iter()
                    .filter(|u| i > 0 || team_houses.is_empty() || team_houses.contains(&u.houses))
                    .choose(&mut thread_rng())
                    .map(|unit| ShopOffer {
                        available: true,
                        unit: TeamUnit { id, unit },
                        discount: (&mut thread_rng())
                            .gen_bool(GlobalSettings::get().discount_chance),
                    })
                    .unwrap(),
            );
        }
    }

    fn position_team(&self, id: u64) -> Result<usize, String> {
        self.state
            .team
            .iter()
            .position(|u| u.id.eq(&id))
            .context_str("Unit not found")
    }
    fn position_case(&self, id: u64) -> Result<usize, String> {
        self.state
            .case
            .iter()
            .position(|u| u.unit.id.eq(&id))
            .context_str("Unit not found")
    }
    fn find_team(&self, id: u64) -> Result<(usize, &TeamUnit), String> {
        let index = self.position_team(id)?;
        Ok((index, &self.state.team[index]))
    }
    fn find_offer(&self, id: u64) -> Result<(usize, &ShopOffer), String> {
        let index = self.position_case(id)?;
        Ok((index, &self.state.case[index]))
    }

    fn buy(
        &mut self,
        id: u64,
        slot: usize,
        price: i64,
        skip_limit_check: bool,
    ) -> Result<(), String> {
        let offer = self
            .state
            .case
            .iter_mut()
            .find(|o| o.unit.id.eq(&id))
            .context_str("Offer not found")?;
        if !offer.available {
            return Err("Offer is already bought".to_owned());
        }
        offer.available = false;
        let offer = offer.clone();
        if !self.can_afford(price) {
            return Err("Not enough g".to_owned());
        }
        if !skip_limit_check && self.state.team.len() >= GlobalSettings::get().team_slots as usize {
            return Err("Team is already full".to_owned());
        }
        self.change_g(-price);
        self.state.team.insert(slot, offer.unit);
        Ok(())
    }

    fn save(mut self) {
        self.last_updated = Timestamp::now();
        Self::update_by_id(&self.id.clone(), self);
    }
}

impl ArenaBattle {
    fn next(run: &ArenaRun) -> Option<Self> {
        ArenaPool::filter_by_round(&run.round)
            .choose(&mut thread_rng())
            .map(|a| ArenaBattle {
                enemy: a.id,
                result: None,
            })
    }
}
