use std::i32;

use rand::{seq::SliceRandom, Rng};

use super::*;

impl NMatch {
    fn fill_shop_case(&mut self, ctx: &ReducerContext) -> Result<(), String> {
        if let Ok(sc) = self.shop_case_load(ctx) {
            for sc in sc {
                sc.delete_self(ctx);
            }
        }
        let gs = ctx.global_settings();

        let unit_price = gs.match_g.unit_buy;
        let house_price = gs.match_g.house_buy;
        let owned_houses: HashSet<String> = HashSet::from_iter(
            self.team_load(ctx)?
                .houses_load(ctx)
                .map(|h| h.into_iter().map(|h| h.house_name.clone()).collect_vec())
                .unwrap_or_default(),
        );
        let all_houses = NHouse::collect_owner(ctx, ID_CORE);
        let not_owned_houses = all_houses
            .iter()
            .filter(|h| !owned_houses.contains(&h.house_name))
            .map(|h| h.id)
            .collect_vec();
        let units_from_owned_houses = all_houses
            .into_iter()
            .filter(|h| owned_houses.contains(&h.house_name))
            .flat_map(|h| h.id.collect_kind_children(ctx, NodeKind::NUnit))
            .collect_vec();
        self.shop_case = (0..4)
            .map(|_| {
                let unit = ctx.rng().gen_bool(0.5);
                let n =
                    if unit && !units_from_owned_houses.is_empty() || not_owned_houses.is_empty() {
                        NShopOffer::new(
                            ctx,
                            self.owner,
                            unit_price,
                            *units_from_owned_houses.choose(&mut ctx.rng()).unwrap(),
                            CardKind::Unit,
                            false,
                        )
                    } else {
                        NShopOffer::new(
                            ctx,
                            self.owner,
                            house_price,
                            *not_owned_houses.choose(&mut ctx.rng()).unwrap(),
                            CardKind::House,
                            false,
                        )
                    };
                n.id.add_parent(ctx, self.id).unwrap();
                n
            })
            .collect_vec();
        Ok(())
    }
}

#[reducer]
fn match_buy(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let g = m.g;
    if m.hand.len() >= 7 {
        return Err("Hand is full".into());
    }
    let sc = m
        .shop_case_load(ctx)?
        .into_iter()
        .find(|s| s.id() == id)
        .to_custom_e_s_fn(|| format!("Shop case slot not found for #{id}"))?;
    if sc.price > g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    let price = sc.price;
    let card_kind = sc.card_kind;
    let id = sc.node_id;
    m.hand.push((card_kind, id));
    m.g -= price;
    // let unit = sc.unit;
    // let unit = NUnit::get(ctx, unit)
    //     .to_custom_e_s_fn(|| format!("Failed to find Unit#{unit}"))?
    //     .with_children(ctx)
    //     .with_components(ctx)
    //     .take();
    // let mut house = unit
    //     .find_parent::<NHouse>(ctx)
    //     .to_custom_e_s("Failed to find House parent of Unit")?;
    // let team = m.team_load(ctx)?;
    // let _ = team.houses_load(ctx);
    // let houses = &mut team.houses;
    // if let Some(h) = houses.iter_mut().find(|h| h.house_name == house.house_name) {
    //     unit.clone(ctx, pid, &mut default())
    //         .id
    //         .add_parent(ctx, h.id)?;
    // } else {
    //     let house = house.with_components(ctx).clone(ctx, pid, &mut default());
    //     house.id.add_parent(ctx, team.id)?;
    //     unit.clone(ctx, pid, &mut default())
    //         .id
    //         .add_parent(ctx, house.id)?;
    // }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_play_card(ctx: &ReducerContext, i: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let i = i as usize;
    let Some((card_kind, id)) = m.hand.get(i).copied() else {
        return Err(format!("Card {i} not found in hand"));
    };
    m.hand.remove(i);
    match card_kind {
        CardKind::Unit => todo!(),
        CardKind::House => {
            let house =
                id.to_node::<NHouse>(ctx)?
                    .with_components(ctx)
                    .clone(ctx, pid, &mut default());
            house.id.add_parent(ctx, m.team_load(ctx)?.id)?;
        }
    }
    m.save(ctx);
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
    unit.delete_with_components(ctx);
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
    m.fill_shop_case(ctx);
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
    let fusion = NFusion::new(ctx, pid, default(), i32::MAX, 0, 0, 0, 1, default());
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
fn match_add_fusion_unit(ctx: &ReducerContext, fusion_id: u64, unit_id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    if !m.roster_units_load(ctx)?.iter().any(|u| u.id == unit_id) {
        return Err(format!("Unit#{unit_id} not found"));
    }
    let fusions = m.team_load(ctx)?.fusions_load(ctx)?;
    if let Some(f) = fusions
        .iter()
        .find(|f| f.units.ids.iter().any(|id| *id == unit_id))
    {
        return Err(format!("Fusion#{} already contains Unit#{unit_id}", f.id));
    }
    let fusion = fusions
        .into_iter()
        .find(|f| f.id == fusion_id)
        .to_custom_e_s_fn(|| format!("Failed to find Fusion#{fusion_id}"))?;
    fusion.units_add(ctx, unit_id)
}

#[reducer]
fn match_remove_fusion_unit(
    ctx: &ReducerContext,
    fusion_id: u64,
    unit_id: u64,
) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let fusion = m
        .team_load(ctx)?
        .fusions_load(ctx)?
        .into_iter()
        .find(|f| f.id == fusion_id)
        .to_custom_e_s_fn(|| format!("Failed to find Fusion#{fusion_id}"))?;
    fusion.units_remove(ctx, unit_id)
}

#[reducer]
fn match_reorder_fusions(ctx: &ReducerContext, fusions: Vec<u64>) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let fusions_n = m.team_load(ctx)?.fusions_load(ctx)?;
    if fusions.len() != fusions_n.len() {
        return Err("Wrong fusions amount".into());
    }
    if let Some(id) = fusions.iter().duplicates().next() {
        return Err(format!("Duplicate Fusion id#{id}"));
    }
    if let Some(f) = fusions_n.iter().find(|f| !fusions.contains(&f.id)) {
        return Err(format!("Fusion#{} is absent in order array", f.id));
    }
    for (i, f) in fusions_n
        .iter_mut()
        .sorted_by_key(|f| fusions.iter().position(|id| f.id.eq(id)).unwrap())
        .enumerate()
    {
        f.slot = i as i32;
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    if let Ok(m) = player.active_match_load(ctx) {
        m.delete_with_components(ctx);
    }
    let gs = ctx.global_settings();
    let mut m = NMatch::new(ctx, pid, gs.match_g.initial, 0, 0, 3, true, default());
    m.id.add_child(ctx, player.id)?;
    let team = NTeam::new(ctx, pid);
    team.id.add_child(ctx, m.id)?;
    m.team = Some(team);
    m.fill_shop_case(ctx);
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
        let new_pool = NFloorPool::new(ctx, ID_ARENA, floor);
        new_pool.id.add_parent(ctx, arena.id)?;
        let id = new_pool.id;
        arena.floor_pools.push(new_pool);
        id
    };
    if let Some(team) = pool_id
        .collect_kind_children(ctx, NodeKind::NTeam)
        .choose(&mut ctx.rng())
        .and_then(|id| id.to_node::<NTeam>(ctx).ok())
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
        let floor_boss = NFloorBoss::new(ctx, ID_ARENA, floor);
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
        m.delete_with_components(ctx);
        Ok(())
    }
}
