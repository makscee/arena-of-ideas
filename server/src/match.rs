use std::i32;

use rand::seq::SliceRandom;

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
        .to_custom_e_s_fn(|| format!("Shop case slot not found for #{id}"))?;
    if sc.price > g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    let unit = sc.unit;
    let price = sc.price;
    m.g -= price;
    let unit = NUnit::get(ctx, unit)
        .to_custom_e_s_fn(|| format!("Failed to find Unit#{unit}"))?
        .with_children(ctx)
        .with_components(ctx)
        .take();
    let mut house = unit.find_parent::<NHouse>(ctx)?;
    let team = m.team_load(ctx)?;
    let _ = team.houses_load(ctx);
    let houses = &mut team.houses;
    if let Some(h) = houses.iter_mut().find(|h| h.house_name == house.house_name) {
        unit.clone(ctx, &mut default()).id.add_parent(ctx, h.id)?;
    } else {
        let house = house.with_components(ctx).clone(ctx, &mut default());
        house.id.add_parent(ctx, team.id)?;
        unit.clone(ctx, &mut default())
            .id
            .add_parent(ctx, house.id)?;
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
        .to_custom_e_s_fn(|| format!("Failed to find unit {name}"))?;
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
    let pid = player.id;
    let team = player.active_match_load(ctx)?.team_load(ctx)?;
    let _ = team.fusions_load(ctx);
    if team.fusions.len() >= ctx.global_settings().team_slots as usize {
        return Err("Team size limit reached".into());
    }
    let fusion = NFusion::new(ctx, pid, i32::MAX, 0, 0, 0, default());
    fusion.id.add_parent(ctx, team.id())?;
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
    let pid = player.id;
    if let Ok(m) = player.active_match_load(ctx) {
        m.delete_recursive(ctx);
    }
    let mut core = NCore::load(ctx);
    let units = core.all_units(ctx)?;
    let gs = ctx.global_settings();
    let price = gs.match_g.unit_buy;
    let mut m = NMatch::new(ctx, pid, gs.match_g.initial, 0, 0, 3, true);
    m.id.add_parent(ctx, player.id)?;
    let team = NTeam::new(ctx, pid);
    team.id.add_parent(ctx, m.id)?;
    m.team = Some(team);
    m.shop_case = (0..3)
        .map(|_| {
            let n = NShopCaseUnit::new(
                ctx,
                pid,
                price,
                units.choose(&mut ctx.rng()).unwrap().id,
                false,
            );
            n.id.add_parent(ctx, m.id).unwrap();
            n
        })
        .collect_vec();
    player.active_match = Some(m);
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let m_id = m.id;
    m.round += 1;
    let floor = m.floor;
    let mut arena = NArena::get(ctx, ID_ARENA).to_custom_e_s("Failed to get Arena")?;
    let _ = arena.floor_pools_load(ctx);
    let player_team = m.team_load(ctx)?.with_children(ctx).with_components(ctx);
    let pool_id = if let Some(pool) = arena.floor_pools.iter().find(|p| p.floor == floor) {
        pool.id
    } else {
        let new_pool = NFloorPool::new(ctx, 0, floor);
        new_pool.id.add_parent(ctx, arena.id)?;
        let id = new_pool.id;
        arena.floor_pools.push(new_pool);
        id
    };
    if let Some(team) = pool_id
        .collect_kind_parents(ctx, NodeKind::NTeam)
        .choose(&mut ctx.rng())
        .and_then(|(id, score)| NTeam::get(ctx, *id))
    {
        let player_team_id = player_team.clone_ids_remap(ctx, pool_id)?.id;
        NBattle::new(
            ctx,
            pid,
            player_team_id,
            team.id,
            ctx.timestamp.to_micros_since_unix_epoch() as u64,
            default(),
            None,
        )
        .id
        .add_parent(ctx, m_id)?;
    } else {
        let _ = arena.floor_bosses_load(ctx);
        let floor_boss = NFloorBoss::new(ctx, 0, floor);
        floor_boss.id.add_parent(ctx, arena.id)?;
        player_team.clone_ids_remap(ctx, floor_boss.id)?;
        player_team.clone_ids_remap(ctx, pool_id)?;
        m.active = false;
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_submit_battle_result(
    ctx: &ReducerContext,
    id: u64,
    result: bool,
    hash: u64,
) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let battle = m.battles_load(ctx)?.last_mut().unwrap();
    if battle.id != id {
        return Err("Wrong Battle id".into());
    }
    if battle.result.is_some() {
        return Err("Battle result already submitted".into());
    }
    battle.result = Some(result);
    battle.hash = hash;
    if result {
        m.floor += 1;
    } else {
        m.lives -= 1;
    }
    if m.lives <= 0 {
        m.active = false;
    }
    m.g += ctx.global_settings().match_g.initial;
    match_reroll(ctx)?;
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_complete(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    if m.active {
        Err("Match is still active".into())
    } else {
        m.delete_recursive(ctx);
        Ok(())
    }
}
