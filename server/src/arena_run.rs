use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use crate::unit::TableUnit;

use super::*;

#[spacetimedb(table)]
pub struct ArenaRun {
    #[primarykey]
    #[autoinc]
    id: u64,
    user_id: u64,
    enemies: Vec<u64>,
    state: GameState,
    active: bool,
}
#[derive(SpacetimeType)]
pub struct GameState {
    wins: u8,
    loses: u8,
    g: i64,
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
    price: i64,
    unit: TeamUnit,
}

const G_PER_ROUND: i64 = 4;
const PRICE_REROLL: i64 = 1;
const PRICE_UNIT: i64 = 3;
const PRICE_SELL: i64 = 1;
const TEAM_SLOTS: usize = 7;

#[spacetimedb(reducer)]
fn run_start(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    if let Some(mut run) = ArenaRun::filter_by_user_id(&user.id).find(|r| r.active) {
        run.active = false;
        run.update();
    }
    let mut run = ArenaRun::new(user.id);
    if let Some(enemy) = ArenaPool::filter_by_round(&0).choose(&mut thread_rng()) {
        run.enemies.push(enemy.id);
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
        round: run.round(),
        team,
    })?;
    if win {
        run.state.wins += 1;
    } else {
        run.state.loses += 1;
    }
    if run.state.loses > 2 || run.state.wins > 9 {
        run.active = false;
    } else {
        let round = run.state.wins + run.state.loses;
        if let Some(enemy) = ArenaPool::filter_by_round(&round)
            .filter(|e| e.owner != user_id)
            .choose(&mut thread_rng())
        {
            run.enemies.push(enemy.id);
        }
    }
    run.change_g(G_PER_ROUND);
    run.state.case.clear();
    run.fill_case();
    run.update();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_reroll(ctx: ReducerContext, force: bool) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    if force || run.can_afford(PRICE_REROLL) {
        if !force {
            run.change_g(-PRICE_REROLL);
        }
        run.state.case.clear();
        run.fill_case();
        run.update();
        Ok(())
    } else {
        Err("Not enough g".to_owned())
    }
}

#[spacetimedb(reducer)]
fn run_buy(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let offer = run
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
    let price = offer.price;
    if !run.can_afford(price) {
        return Err("Not enough g".to_owned());
    }
    if run.state.team.len() >= TEAM_SLOTS {
        return Err("Team is already full".to_owned());
    }
    run.change_g(-PRICE_UNIT);
    run.state.team.insert(0, offer.unit);
    run.update();
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
    run.change_g(PRICE_SELL);
    run.update();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_stack(ctx: ReducerContext, target: u64, dragged: u64) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    let (i_target, target) = run.find_team(target)?;
    let (i_dragged, dragged) = run.find_team(dragged)?;
    if !target.unit.house.eq(&dragged.unit.house) {
        return Err("Houses should match for stacking".to_owned());
    }
    let target = &mut run.state.team[i_target].unit;
    target.atk += 1;
    target.hp += 1;
    target.stacks += 1;
    if target.stacks >= target.level + 1 {
        target.stacks -= target.level;
        target.level += 1;
    }
    run.state.team.remove(i_dragged);
    run.update();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_team_reorder(ctx: ReducerContext, order: Vec<u64>) -> Result<(), String> {
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    run.state
        .team
        .sort_by_key(|u| order.iter().position(|o| u.id.eq(o)));
    run.update();
    Ok(())
}

#[spacetimedb(reducer)]
fn run_change_g(ctx: ReducerContext, delta: i64) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    let (_, mut run) = ArenaRun::get_by_identity(&ctx.sender)?;
    run.change_g(delta);
    run.update();
    Ok(())
}

impl ArenaRun {
    fn new(user_id: u64) -> Self {
        Self {
            user_id,
            active: true,
            id: 0,
            enemies: Vec::default(),
            state: GameState {
                wins: 0,
                loses: 0,
                g: G_PER_ROUND,
                team: Vec::default(),
                case: Vec::default(),
                next_id: 0,
            },
        }
    }

    fn round(&self) -> u8 {
        self.state.loses + self.state.wins
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
        for _ in 0..3 {
            let id = self.next_id();
            self.state.case.push(
                TableUnit::iter()
                    .choose(&mut thread_rng())
                    .map(|unit| ShopOffer {
                        available: true,
                        price: PRICE_UNIT,
                        unit: TeamUnit { id, unit },
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

    fn find_team_mut(&mut self, id: u64) -> Result<(usize, &mut TeamUnit), String> {
        let index = self.position_team(id)?;
        Ok((index, &mut self.state.team[index]))
    }
    fn find_team(&self, id: u64) -> Result<(usize, &TeamUnit), String> {
        let index = self.position_team(id)?;
        Ok((index, &self.state.team[index]))
    }

    fn update(self) {
        Self::update_by_id(&self.id.clone(), self);
    }
}
