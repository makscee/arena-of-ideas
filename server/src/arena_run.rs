use rand::seq::IteratorRandom;
use rand::thread_rng;

use super::*;

#[spacetimedb(table)]
pub struct ArenaRun {
    #[primarykey]
    #[autoinc]
    id: u64,
    user: u64,
    wins: u8,
    loses: u8,
    enemies: Vec<u64>,
    active: bool,
}

#[spacetimedb(reducer)]
fn start_run(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    if let Some(mut run) = ArenaRun::filter_by_user(&user.id).find(|r| r.active) {
        run.active = false;
        ArenaRun::update_by_id(&run.id.clone(), run);
    }
    let mut run = ArenaRun::new(user.id);
    if let Some(enemy) = ArenaPool::filter_by_round(&0).choose(&mut thread_rng()) {
        run.enemies.push(enemy.id);
    }
    Ok(())
}
#[spacetimedb(reducer)]
fn submit_run_result(ctx: ReducerContext, team: String, win: bool) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    let mut run = ArenaRun::filter_by_user(&user.id)
        .find(|r| r.active)
        .context("No arena run in progress")
        .map_err(|e| e.to_string())?;
    ArenaPool::insert(ArenaPool {
        id: 0,
        owner: user.id,
        round: run.wins + run.loses,
        team,
    })?;
    if win {
        run.wins += 1;
    } else {
        run.loses += 1;
    }
    if run.loses > 2 || run.wins > 9 {
        run.active = false;
    } else {
        let round = run.wins + run.loses;
        if let Some(enemy) = ArenaPool::filter_by_round(&round)
            .filter(|e| e.owner != user.id)
            .choose(&mut thread_rng())
        {
            run.enemies.push(enemy.id);
        }
    }
    ArenaRun::update_by_id(&run.id.clone(), run);
    Ok(())
}

impl ArenaRun {
    fn new(user: u64) -> Self {
        Self {
            user,
            wins: 0,
            loses: 0,
            active: true,
            id: 0,
            enemies: Vec::default(),
        }
    }
}
