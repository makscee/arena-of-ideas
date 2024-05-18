use std::collections::HashSet;

use itertools::Itertools;
use rand::distributions::WeightedIndex;
use rand::seq::IteratorRandom;
use rand::{distributions::Distribution, thread_rng, Rng};
use spacetimedb::Timestamp;

use crate::unit::TableUnit;

use super::*;

#[spacetimedb(table)]
pub struct ArenaRun {
    #[primarykey]
    id: u64,
    #[unique]
    user_id: u64,
    battles: Vec<ArenaBattle>,
    round: u32,
    state: RunState,
    last_updated: Timestamp,
}
#[derive(SpacetimeType)]
pub struct ArenaBattle {
    enemy: u64,
    result: Option<bool>,
}
#[derive(SpacetimeType)]
pub struct RunState {
    g: i32,
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
    freeze: bool,
    unit: TeamUnit,
}

#[spacetimedb(reducer)]
fn run_start(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    ArenaRun::delete_by_user_id(&user.id);
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
    if !team.is_empty() {
        ArenaPool::insert(ArenaPool {
            id: GlobalData::next_id(),
            owner: user_id,
            round: run.round,
            team,
        })?;
    }
    let mut finish = false;
    if let Some(last_battle) = run.battles.get_mut(run.round as usize) {
        last_battle.result = Some(win);
    } else {
        finish = true;
    }
    run.round += 1;

    if run.loses() > 2 {
        finish = true;
    } else {
        if let Some(enemy) = ArenaBattle::next(&run) {
            run.battles.push(enemy);
        }
    }
    if !finish {
        let settings = GlobalSettings::get();
        run.change_g((settings.g_per_round_min + run.round as i32).min(settings.g_per_round_max));
        run.state.case.retain(|o| o.freeze && o.available);
        run.fill_case();
        run.state.free_rerolls = 1;
    }
    if finish {
        run.finish();
    } else {
        run.save();
    }
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
        run.state.case.retain(|o| o.freeze && o.available);
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
    let (_, offer) = run.find_offer(id)?;
    let mut price = gs.rarities.prices[offer.unit.unit.rarity as usize];
    if offer.discount {
        price = (price as f32 * gs.price_unit_discount) as i32;
    }
    run.buy(id, 0, price, false)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_freeze(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    run.freeze(id)?;
    run.save();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_sell(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let (index, unit) = run.find_team(id)?;
    let gs = GlobalSettings::get();
    run.change_g(
        (gs.rarities.prices[unit.unit.rarity as usize] as f32 * gs.price_unit_sell) as i32,
    );
    run.state.team.remove(index);
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

        let (_, offer) = run.find_offer(source)?;
        let mut price = gs.rarities.prices[offer.unit.unit.rarity as usize];
        let mul = if offer.discount {
            gs.price_unit_discount
        } else {
            gs.price_unit_buy_stack
        };
        price = (price as f32 * mul) as i32;
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
fn run_change_g(ctx: ReducerContext, delta: i32) -> Result<(), String> {
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
            id: GlobalData::next_id(),
            user_id,
            last_updated: Timestamp::now(),
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

    fn wins(&self) -> u32 {
        self.battles
            .iter()
            .filter(|b| b.result.is_some_and(|r| r))
            .count() as u32
    }

    fn get_by_identity(identity: &Identity) -> Result<(u64, Self), String> {
        let user_id = User::find_by_identity(identity)?.id;
        Ok((user_id, Self::get_active(&user_id)?))
    }

    fn get_active(user_id: &u64) -> Result<Self, String> {
        ArenaRun::filter_by_user_id(user_id).context_str("No arena run in progress")
    }

    fn can_afford(&self, price: i32) -> bool {
        self.state.g >= price
    }
    fn change_g(&mut self, delta: i32) {
        self.state.g += delta;
    }

    fn next_id(&mut self) -> u64 {
        self.state.next_id += 1;
        self.state.next_id
    }

    fn get_weight(&self, unit: &TableUnit) -> i32 {
        let gs = GlobalSettings::get();
        let round = self.round as i32;
        let rarity = unit.rarity as usize;
        (gs.rarities.weights_initial[rarity] + gs.rarities.weights_per_round[rarity] * round).max(0)
    }

    fn fill_case(&mut self) {
        let settings = GlobalSettings::get();
        let slots = (settings.shop_slots_min
            + (self.round as f32 * settings.shop_slots_per_round) as u32)
            .min(settings.shop_slots_max)
            - self.state.case.len() as u32;
        let team_houses: HashSet<String> = HashSet::from_iter(
            self.state
                .team
                .iter()
                .map(|u| u.unit.houses.split("+"))
                .flatten()
                .map(|s| s.to_owned())
                .collect_vec(),
        );
        let items = TableUnit::iter()
            .map(|u| {
                let weight = self.get_weight(&u) as f32;
                (u, weight)
            })
            .collect_vec();
        let family_items = TableUnit::iter()
            .filter(|u| team_houses.is_empty() || team_houses.contains(&u.houses))
            .map(|u| {
                let weight = self.get_weight(&u) as f32;
                (u, weight)
            })
            .collect_vec();

        let dist = WeightedIndex::new(items.iter().map(|item| item.1)).unwrap();
        let dist_family = WeightedIndex::new(family_items.iter().map(|item| item.1)).unwrap();
        for i in 0..slots {
            let id = self.next_id();
            let unit = if i == 0 {
                family_items[dist_family.sample(&mut thread_rng())]
                    .0
                    .clone()
            } else {
                items[dist.sample(&mut thread_rng())].0.clone()
            };
            let unit = ShopOffer {
                available: true,
                freeze: false,
                unit: TeamUnit { id, unit },
                discount: (&mut thread_rng()).gen_bool(GlobalSettings::get().discount_chance),
            };
            self.state.case.push(unit);
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
    fn find_offer_mut(&mut self, id: u64) -> Result<(usize, &mut ShopOffer), String> {
        let index = self.position_case(id)?;
        Ok((index, &mut self.state.case[index]))
    }

    fn buy(
        &mut self,
        id: u64,
        slot: usize,
        price: i32,
        skip_limit_check: bool,
    ) -> Result<(), String> {
        let (_, offer) = self.find_offer_mut(id)?;
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

    fn freeze(&mut self, id: u64) -> Result<(), String> {
        let (_, offer) = self.find_offer_mut(id)?;
        offer.freeze = !offer.freeze;
        Ok(())
    }

    fn save(mut self) {
        self.last_updated = Timestamp::now();
        Self::update_by_user_id(&self.user_id.clone(), self);
    }

    fn finish(self) {
        ArenaRun::delete_by_id(&self.id);
        let archive = ArenaArchive {
            id: self.id,
            user_id: self.user_id,
            round: self.round,
            wins: self.wins(),
            loses: self.loses(),
            season: GlobalSettings::get().season,
            team: self.state.team.into_iter().map(|u| u.unit).collect_vec(),
            timestamp: self.last_updated,
        };
        ArenaArchive::insert(archive).unwrap();
    }
}

impl ArenaBattle {
    fn next(run: &ArenaRun) -> Option<Self> {
        if ArenaPool::filter_by_round(&(run.round + 1)).count() == 0 {
            ArenaPool::filter_by_round(&run.round).min_by_key(|r| r.id)
        } else {
            ArenaPool::filter_by_round(&run.round).choose(&mut thread_rng())
        }
        .map(|a| ArenaBattle {
            enemy: a.id,
            result: None,
        })
    }
}
