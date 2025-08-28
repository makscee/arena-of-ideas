use rand::seq::SliceRandom;

use super::*;

#[reducer]
fn match_shop_buy(ctx: &ReducerContext, shop_idx: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let offer = m
        .shop_offers
        .last_mut()
        .to_custom_e_s("No active shop offers")?;
    let slot = offer
        .case
        .get_mut(shop_idx as usize)
        .to_custom_e_s_fn(|| format!("Shop slot {shop_idx} not found"))?;
    if slot.sold {
        return Err("Shop slot already sold".to_string());
    }
    slot.sold = true;
    let price = slot.price;
    let node_id = slot.node_id;
    match slot.card_kind {
        CardKind::Unit => {
            let unit = NUnit::get(ctx, node_id)
                .to_custom_e_s_fn(|| format!("Failed to find Unit#{node_id}"))?
                .with_parts(ctx)
                .take();
            let house_id = unit
                .id
                .find_kind_parent(ctx, NodeKind::NHouse)
                .to_custom_e_s("NHouse parent of NUnit not found")?;
            let house_name = house_id.load_node::<NHouse>(ctx)?.house_name;
            let house = m
                .team_load(ctx)?
                .houses_load(ctx)?
                .iter()
                .find(|h| h.house_name == house_name)
                .to_custom_e_s_fn(|| format!("House {house_name} not found"))?;
            let mut unit = unit.clone(ctx, pid);
            unit.state_set(ctx, NUnitState::new_full(pid, 1).insert(ctx))?;
            unit.id.add_parent(ctx, house.id)?;
        }
        CardKind::House => {
            let house = NHouse::loader(node_id)
                .with_all_parts()
                .without_units()
                .load(ctx)?;
            let _ = m.team_load(ctx)?.houses_load(ctx);
            if m.team_load(ctx)?
                .houses
                .get_data()
                .to_custom_e_s("Houses not loaded")?
                .iter()
                .any(|h| h.house_name == house.house_name)
            {
                // increase house lvl
            } else {
                let house = house.clone(ctx, pid);
                house.id.add_parent(ctx, m.team_load(ctx)?.id)?;
            }
            m.team.set_unknown();
            m.fill_shop_case(ctx, true)?;
        }
    }
    m.buy(ctx, price)?;
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_move_unit(ctx: &ReducerContext, unit_id: u64, target_id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let mut unit = unit_id.load_node::<NUnit>(ctx)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".into());
    }
    let m = player.active_match_load(ctx)?;
    let old_target_id = m.unlink_unit(ctx, unit_id);
    if old_target_id.is_some_and(|id| id == target_id) {
        return Err("Unit already at target".into());
    }
    let target_node = target_id
        .load_tnode(ctx)
        .to_custom_e_s_fn(|| format!("Failed to find Node#{target_id}"))?;
    if target_node.owner != pid {
        return Err("Target node not owned by player".into());
    }
    if let Some(mut slot_unit) = target_id
        .get_kind_parent(ctx, NodeKind::NUnit)
        .and_then(|id| id.load_node::<NUnit>(ctx).ok())
    {
        if slot_unit.unit_name == unit.unit_name {
            let s_state = slot_unit.state_load(ctx)?;
            let u_state = unit.state_load(ctx)?;
            s_state.stacks += u_state.stacks;
            unit.delete_with_parts(ctx);
            s_state.save(ctx);
        } else {
            m.unlink_unit(ctx, slot_unit.id);
            unit_id.add_child(ctx, target_id)?;
            if let Some(old_target_id) = old_target_id {
                slot_unit.id.add_child(ctx, old_target_id)?;
            }
        }
    } else {
        unit_id.add_child(ctx, target_id)?;
    }
    Ok(())
}

#[reducer]
fn match_sell_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let unit = unit_id.load_node::<NUnit>(ctx)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let m = player.active_match_load(ctx)?;
    m.g += ctx.global_settings().match_g.unit_sell;
    unit.delete_with_parts(ctx);
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_bench_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let unit = unit_id.load_node::<NUnit>(ctx)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let m = player.active_match_load(ctx)?;
    m.unlink_unit(ctx, unit_id);
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_buy_fusion_slot(ctx: &ReducerContext, fusion_id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let mut fusion = fusion_id.load_node::<NFusion>(ctx)?;
    let slots = fusion.slots_load(ctx)?;
    let price = ctx.global_settings().match_g.fusion_slot_mul * slots.len() as i32;
    let fs = NFusionSlot::new(pid, slots.len() as i32, default()).insert(ctx);
    fs.id.add_child(ctx, fusion.id)?;
    m.buy(ctx, price)
}

#[reducer]
fn match_shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.fill_shop_case(ctx, true)?;
    m.buy(ctx, ctx.global_settings().match_g.reroll)
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
    m.floor += 1;
    let battle = m.battles_load(ctx)?.last_mut().unwrap();
    if battle.id != id {
        return Err("Wrong Battle id".into());
    }
    if battle.result.is_some() {
        return Err("Battle result already submitted".into());
    }
    battle.result = Some(result);
    battle.hash = hash;
    if !result {
        m.lives -= 1;
    }
    if m.lives <= 0 {
        m.active = false;
    }
    m.g += ctx.global_settings().match_g.initial;
    m.fill_shop_case(ctx, false)?;
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let m_id = m.id;
    let floor = m.floor;
    let mut arena = NArena::loader(ID_ARENA)
        .with_floor_pools()
        .with_floor_bosses()
        .load(ctx)?;
    let player_team = m.team_load(ctx)?.with_parts(ctx).take().clone(ctx, pid);
    let pool_id = if let Some(pool) = arena.floor_pools.iter().find(|p| p.floor == floor) {
        pool.id
    } else {
        let new_pool = NFloorPool::new(ID_ARENA, floor).insert(ctx);
        let id = new_pool.id;
        arena.floor_pools_add(ctx, new_pool)?;
        id
    };
    let teams = pool_id.collect_kind_children(ctx, NodeKind::NTeam);
    player_team.id.add_parent(ctx, pool_id)?;
    let enemy_team = if let Some(team_id) = teams.choose(&mut ctx.rng()) {
        team_id.load_node::<NTeam>(ctx)?
    } else {
        let mut floor_boss = NFloorBoss::new(ID_ARENA, floor).insert(ctx);
        player_team.id.add_parent(ctx, floor_boss.id)?;
        arena.floor_bosses_add(ctx, floor_boss)?;
        team
    };
    {
        let player_team_id = player_team.clone(ctx, pid).id;
        player_team_id.add_parent(ctx, pool_id)?;
        let battle = NBattle::new(
            pid,
            player_team_id,
            enemy_team.id,
            ctx.timestamp.to_micros_since_unix_epoch() as u64,
            default(),
            None,
        )
        .insert(ctx);
        battle.id.add_parent(ctx, m_id)?;
    }
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_complete(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    if m.active {
        Err("Match is still active".into())
    } else {
        m.delete_with_parts(ctx);
        Ok(())
    }
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    for m in NMatch::collect_owner(ctx, player.id) {
        m.delete_with_parts(ctx);
    }
    let gs = ctx.global_settings();
    let mut m = NMatch::new(pid, gs.match_g.initial, 0, 3, true, default()).insert(ctx);
    let mut team = NTeam::new(pid).insert(ctx);
    for i in 0..ctx.global_settings().team_slots as i32 {
        let mut fusion = NFusion::new(pid, default(), i, 0, 0, 0, 1).insert(ctx);
        let slot = NFusionSlot::new(pid, 0, default()).insert(ctx);
        fusion.slots_add(ctx, slot)?;
        team.fusions_add(ctx, fusion)?;
    }
    m.team_set(ctx, team)?;
    m.fill_shop_case(ctx, false)?;
    player.active_match_set(ctx, m)?;
    player.save(ctx);
    Ok(())
}

impl NMatch {
    fn buy(&mut self, ctx: &ReducerContext, price: i32) -> Result<(), String> {
        if self.g < price {
            return Err(format!(
                "Can't afford: price = {price} match g = {}",
                self.g
            ));
        }
        self.g -= price;
        self.save(ctx);
        Ok(())
    }
    fn unlink_unit(&mut self, ctx: &ReducerContext, unit_id: u64) -> Option<u64> {
        let links = TNodeLink::children_of_kind(ctx, unit_id, NodeKind::NFusionSlot, true);
        if links.len() > 1 {
            error!("Unit#{} linked to {} slots", unit_id, links.len());
        }
        let res = links.first().map(|l| l.child);
        for link in links {
            ctx.db.node_links().delete(link);
        }
        res
    }
    fn fill_shop_case(&mut self, ctx: &ReducerContext, units: bool) -> Result<(), String> {
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
        let shop_case = (0..4)
            .map(|_| {
                let n = if units && !units_from_owned_houses.is_empty() {
                    ShopSlot {
                        card_kind: CardKind::Unit,
                        node_id: *units_from_owned_houses.choose(&mut ctx.rng()).unwrap(),
                        sold: false,
                        price: unit_price,
                        buy_text: None,
                    }
                } else {
                    ShopSlot {
                        card_kind: CardKind::House,
                        node_id: *not_owned_houses.choose(&mut ctx.rng()).unwrap(),
                        sold: false,
                        price: house_price,
                        buy_text: None,
                    }
                };
                n
            })
            .collect_vec();
        self.shop_offers = [ShopOffer {
            buy_limit: None,
            case: shop_case,
        }]
        .into();
        Ok(())
    }
}
