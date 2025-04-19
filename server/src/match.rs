use std::i32;

use rand::seq::{IteratorRandom, SliceRandom};

use super::*;

#[reducer]
fn match_buy(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let g = m.g;
    let sc = m
        .shop_case_load(ctx)?
        .into_iter()
        .find(|s| s.id() == id)
        .to_e_s_fn(|| format!("Shop case slot not found for #{id}"))?;
    if sc.price > g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    let unit = sc.unit;
    let price = sc.price;
    m.g -= price;
    let unit = NUnit::get(ctx, unit)
        .to_e_s_fn(|| format!("Failed to find Unit#{unit}"))?
        .with_children(ctx)
        .with_components(ctx)
        .take();
    let mut house = unit.find_parent::<NHouse>(ctx)?;
    let team = m.team_load(ctx)?;
    let _ = team.houses_load(ctx);
    let houses = &mut team.houses;
    if let Some(h) = houses.iter_mut().find(|h| h.house_name == house.house_name) {
        unit.clone(ctx, h.id, &mut default());
    } else {
        let house = house
            .with_components(ctx)
            .clone(ctx, team.id, &mut default());
        unit.clone(ctx, house.id, &mut default());
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.g += ctx.global_settings().match_g.unit_sell;
    let unit = m
        .roster_units_load(ctx)?
        .into_iter()
        .find(|u| u.unit_name == name)
        .to_e_s_fn(|| format!("Failed to find unit {name}"))?;
    unit.delete_recursive(ctx);
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let cost = ctx.global_settings().match_g.reroll;
    if m.g < cost {
        return Err("Not enough g".into());
    }
    m.g -= cost;
    let sc = m.shop_case_load(ctx)?;
    let mut core = NCore::load(ctx);
    let units = core.all_units(ctx)?;
    for c in sc {
        c.sold = false;
        c.unit = units.choose(&mut ctx.rng()).unwrap().id;
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_buy_fusion(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let team = player.active_match_load(ctx)?.team_load(ctx)?;
    let _ = team.fusions_load(ctx);
    if team.fusions.len() >= ctx.global_settings().team_slots as usize {
        return Err("Team size limit reached".into());
    }
    let fusion = NFusion::new(ctx, team.id(), i32::MAX, default(), default());
    team.fusions.push(fusion);
    for (i, fusion) in team
        .fusions
        .iter_mut()
        .sorted_by_key(|f| f.slot)
        .enumerate()
    {
        fusion.slot = i as i32;
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_edit_fusion(ctx: &ReducerContext, fusion: TNode) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let fusion: NFusion = fusion.to_node()?;
    let team = m.team_load(ctx)?;
    for f in team.fusions_load(ctx)? {
        if f.slot == fusion.slot {
            *f = fusion;
            break;
        }
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    if let Ok(m) = player.active_match_load(ctx) {
        m.delete_recursive(ctx);
    }
    let mut core = NCore::load(ctx);
    let units = core.all_units(ctx)?;
    let gs = ctx.global_settings();
    let price = gs.match_g.unit_buy;
    let mut m = NMatch::new(ctx, player.id, gs.match_g.initial, 0, 0, 3, true);
    let team = NTeam::new(ctx, m.id, player.id);
    m.team = Some(team);
    m.shop_case = (0..3)
        .map(|_| {
            NShopCaseUnit::new(
                ctx,
                m.id,
                price,
                units.choose(&mut ctx.rng()).unwrap().id,
                false,
            )
        })
        .collect_vec();
    player.active_match = Some(m);
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let m_id = m.id;
    m.round += 1;
    let floor = m.floor;
    let mut arena = NArena::get(ctx, ID_ARENA).to_e_s("Failed to get Arena")?;
    let _ = arena.floor_pools_load(ctx);
    let player_team = m.team_load(ctx)?.with_children(ctx).with_components(ctx);
    let pool_id = if let Some(pool) = arena.floor_pools.iter().find(|p| p.floor == floor) {
        pool.id
    } else {
        let new_pool = NFloorPool::new(ctx, arena.id, floor);
        let id = new_pool.id;
        arena.floor_pools.push(new_pool);
        id
    };
    if let Some(team) = ctx
        .db
        .nodes_world()
        .parent()
        .filter(pool_id)
        .choose(&mut ctx.rng())
    {
        NBattle::new(
            ctx,
            m_id,
            player_team.id,
            team.id,
            ctx.timestamp.to_micros_since_unix_epoch() as u64,
            default(),
            None,
        );
    } else {
        let _ = arena.floor_bosses_load(ctx);
        let floor_boss = NFloorBoss::new(ctx, arena.id, floor);
        player_team.clone(ctx, floor_boss.id, &mut default());
    }
    player_team.clone_ids_remap(ctx, pool_id);
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_submit_battle_result(ctx: &ReducerContext, result: bool, hash: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let battle = m.battles_load(ctx)?.last_mut().unwrap();
    if battle.result.is_some() {
        return Err("Battle result already submitted".into());
    }
    battle.result = Some(result);
    battle.hash = hash;
    m.round += 1;
    if result {
        m.floor += 1;
    } else {
        m.lives -= 1;
    }
    if m.lives <= 0 {
        m.active = false;
    }
    player.save(ctx);
    Ok(())
}
