use spacetimedb::rand::seq::SliceRandom;

use super::*;

fn get_floor_pools(ctx: &ServerContext) -> Vec<TNode> {
    TNode::collect_kind_owner(ctx.rctx(), NodeKind::NFloorPool, ID_ARENA)
}

fn get_floor_bosses(ctx: &ServerContext) -> Vec<TNode> {
    TNode::collect_kind_owner(ctx.rctx(), NodeKind::NFloorBoss, ID_ARENA)
}

fn get_last_floor(ctx: &ServerContext) -> i32 {
    TNode::load(ctx.rctx(), ID_ARENA)
        .and_then(|arena| arena.to_node::<NArena>().ok())
        .map(|arena| arena.floors as i32)
        .unwrap_or(0)
}

fn get_floor_pool_teams(ctx: &ServerContext, pool_id: u64) -> Vec<u64> {
    ctx.collect_kind_children(pool_id, NodeKind::NTeam).unwrap()
}

fn get_floor_boss_team_id(ctx: &ServerContext, floor: i32) -> Option<u64> {
    get_floor_bosses(ctx)
        .iter()
        .find_map(|node| {
            let node = node.to_node::<NFloorBoss>().ok()?;
            if node.floor == floor {
                return Some(node);
            } else {
                None
            }
        })
        .and_then(|mut boss| boss.team_load_id(ctx).ok())
}

fn ensure_floor_pool(ctx: &mut ServerContext, floor: i32) -> Result<u64, String> {
    if let Some(pool) = get_floor_pools(ctx).iter().find(|node| {
        node.to_node::<NFloorPool>()
            .map(|pool| pool.floor == floor)
            .unwrap_or(false)
    }) {
        Ok(pool.id)
    } else {
        let new_pool = NFloorPool::new(ctx.next_id(), ID_ARENA, floor).insert(ctx);
        ctx.add_link(ID_ARENA, new_pool.id)?;
        Ok(new_pool.id)
    }
}

fn add_team_to_pool(
    ctx: &mut ServerContext,
    pool_id: u64,
    team: &NTeam,
    owner: u64,
) -> Result<u64, String> {
    let team = team.clone().remap_ids(ctx).with_owner(owner);
    let pool_team_id = team.id;
    team.save(ctx)?;
    ctx.add_link(pool_id, pool_team_id)?;
    Ok(pool_team_id)
}

fn create_battle(
    ctx: &mut ServerContext,
    m: &mut NMatch,
    player_id: u64,
    player_team_id: u64,
    enemy_team_id: u64,
) -> Result<NBattle, String> {
    let battle = NBattle::new(
        ctx.next_id(),
        player_id,
        player_team_id,
        enemy_team_id,
        ctx.rctx().timestamp.to_micros_since_unix_epoch() as u64,
        default(),
        None,
    )
    .insert(ctx);
    ctx.add_link(m.id, battle.id)?;
    m.set_pending_battle(true);

    Ok(battle)
}

#[reducer]
fn match_shop_buy(ctx: &ReducerContext, shop_idx: u8) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?.load_all(ctx)?;
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
    let card_kind = slot.card_kind;
    match card_kind {
        CardKind::Unit => {
            let unit = NUnit::load(ctx.source(), node_id)?
                .load_components(ctx)?
                .take();
            let house_id = unit
                .id
                .get_kind_parent(ctx.rctx(), NodeKind::NHouse)
                .to_not_found()?;
            let house_name = ctx.load::<NHouse>(house_id)?.house_name;
            let house = m
                .team()?
                .houses()?
                .iter()
                .find(|h| h.house_name == house_name)
                .to_custom_e_s_fn(|| format!("House {house_name} not found"))?;
            let mut unit = unit.remap_ids(ctx).with_owner(pid);
            unit.state_set(NUnitState::new(ctx.next_id(), pid, 1))?;
            let unit_id = unit.id;
            unit.save(ctx)?;
            unit_id.add_parent(ctx.rctx(), house.id)?;
        }
        CardKind::House => {
            let house = ctx.load::<NHouse>(node_id)?.load_components(ctx)?.take();
            if m.team()?
                .houses()?
                .iter()
                .any(|h| h.house_name == house.house_name)
            {
                // increase house lvl
            } else {
                let house = house.remap_ids(ctx).with_owner(pid);
                m.team_mut()?.houses_push(house)?;
            }
        }
    }
    m.buy(ctx, price).track()?;
    let mid = m.id;
    debug!("{m:?}");
    m.take().save(ctx).track()?;
    if card_kind == CardKind::House {
        let mut m = ctx.load::<NMatch>(mid)?;
        m.fill_shop_case(ctx, true).track()?;
        m.save(ctx).track()?;
    }
    Ok(())
}

#[reducer]
fn match_move_unit(ctx: &ReducerContext, unit_id: u64, target_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".into());
    }
    let m = player.active_match_load(ctx)?;
    let old_target_id = m.unlink_unit(ctx, unit_id);
    if old_target_id.is_some_and(|id| id == target_id) {
        return Err("Unit already at target".into());
    }
    let target_node = target_id
        .load_tnode(ctx.rctx())
        .to_custom_e_s_fn(|| format!("Failed to find Node#{target_id}"))?;
    if target_node.owner != pid {
        return Err("Target node not owned by player".into());
    }
    let mut slot = target_node.to_node::<NFusionSlot>()?;
    if let Ok(prev_unit) = slot.unit_load(ctx) {
        m.unlink_unit(ctx, prev_unit.id);
        if let Some(prev_slot) = old_target_id {
            prev_slot.add_child(ctx.rctx(), prev_unit.id)?;
        }
    }
    slot.with_unit_id(unit.id).save(ctx)?;
    Ok(())
}

#[reducer]
fn match_sell_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let m = player.active_match_load(ctx)?;
    m.g += ctx.global_settings().match_g.unit_sell;
    unit.delete_recursive(ctx);
    m.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_bench_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let m = player.active_match_load(ctx)?;
    m.unlink_unit(ctx, unit_id);
    m.take().save(ctx).to_server_result()
}

#[reducer]
fn match_buy_fusion_slot(ctx: &ReducerContext, fusion_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let mut fusion = ctx.load::<NFusion>(fusion_id)?;
    let slots = fusion.slots_load(ctx)?;
    let price = ctx.global_settings().match_g.fusion_slot_mul * slots.len() as i32;
    let fs = NFusionSlot::new(ctx.next_id(), pid, slots.len() as i32, default());
    fusion.slots_push(fs)?;
    fusion.save(ctx)?;
    m.buy(ctx, price)?;
    Ok(())
}

#[reducer]
fn match_shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.fill_shop_case(ctx, true)?;
    m.buy(ctx, ctx.global_settings().match_g.reroll)?;
    player.save(ctx)?;
    Ok(())
}

#[reducer]
fn match_submit_battle_result(
    ctx: &ReducerContext,
    id: u64,
    result: bool,
    hash: u64,
) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let current_floor = m.floor;

    let battles = m.battles_load(ctx)?;
    battles.sort_by_key(|b| b.id);
    let battle = battles.last_mut().unwrap();
    debug!("Submit result: flr={current_floor} {battle:?}");
    if battle.id != id {
        return Err("Wrong Battle id".into());
    }
    if battle.result.is_some() {
        return Err("Battle result already submitted".into());
    }
    battle.result = Some(result);
    battle.hash = hash;
    let enemy_team_id = battle.team_right;

    // Clear pending_battle flag on the match
    m.pending_battle = false;

    let last_floor = get_last_floor(ctx);

    // Check if enemy was a boss
    let was_boss_battle = get_floor_boss_team_id(ctx, current_floor) == Some(enemy_team_id);

    // Handle boss battle outcomes
    if was_boss_battle {
        if result {
            // Won against boss - replace boss with player's team
            let player_team_id = m.team_load(ctx)?.id;
            let player_team = ctx.load::<NTeam>(player_team_id)?.load_all(ctx)?.take();

            // Create new boss team
            let new_boss_team = player_team.clone().remap_ids(ctx).with_owner(pid);

            // Replace or create boss entry
            let boss_nodes = get_floor_bosses(ctx);
            if let Some(boss_node) = boss_nodes.iter().find(|n| {
                n.to_node::<NFloorBoss>()
                    .map(|boss| boss.floor == current_floor)
                    .unwrap_or(false)
            }) {
                let mut boss = ctx.load::<NFloorBoss>(boss_node.id)?;
                // Delete old boss team
                if let Ok(old_team) = boss.team_load(ctx) {
                    old_team.clone().delete_recursive(ctx);
                }
                boss.team_set(new_boss_team)?;
                boss.save(ctx)?;
            } else {
                let mut floor_boss = NFloorBoss::new(ctx.next_id(), ID_ARENA, current_floor);
                floor_boss.team_set(new_boss_team)?;
                floor_boss.save(ctx)?;
            }

            // Special case: beating the last floor boss makes you champion
            if current_floor == last_floor {
                // Create a new floor with player as boss
                m.floor += 1;

                let champion_team = player_team.clone().remap_ids(ctx).with_owner(pid);

                let mut new_floor_boss = NFloorBoss::new(ctx.next_id(), ID_ARENA, m.floor);
                new_floor_boss.team_set(champion_team)?;
                new_floor_boss.save(ctx)?;

                // Update arena floors count
                if let Some(mut arena) =
                    TNode::load(ctx.rctx(), ID_ARENA).and_then(|node| node.to_node::<NArena>().ok())
                {
                    arena.floors = m.floor as u8;
                    arena.save(ctx)?;
                }

                // Continue to shop phase
                m.g += ctx.global_settings().match_g.initial;
                m.fill_shop_case(ctx, false)?;
            } else {
                // Regular boss defeat - end the run
                m.active = false;
            }
        } else {
            // Lost against boss - end the run
            m.active = false;
        }
    } else {
        // Regular battle
        if result {
            // Won regular battle
            m.floor += 1;

            // Gain life every 5 floors
            if m.floor % 5 == 0 {
                m.lives += 1;
            }

            m.g += ctx.global_settings().match_g.initial;
            m.fill_shop_case(ctx, false)?;
        } else {
            // Lost regular battle
            m.lives -= 1;

            if m.lives <= 0 {
                m.active = false;
            } else {
                // Continue on same floor
                m.g += ctx.global_settings().match_g.initial;
                m.fill_shop_case(ctx, false)?;
            }
        }
    }
    player.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let m_id = m.id;
    let floor = m.floor;
    let last_floor = get_last_floor(ctx);
    // Regular battles not allowed on last floor
    if floor == last_floor && last_floor > 0 {
        return Err(
            "Regular battles not allowed on the last floor. Must fight the boss!".to_string(),
        );
    }
    // Load player team
    let player_team = m.team_load(ctx)?.load_all(ctx)?;
    // Ensure floor pool exists and add player's team to it
    let pool_id = ensure_floor_pool(ctx, floor)?;
    // Get random enemy from pool or use team id 0 if empty
    let pool_teams = get_floor_pool_teams(ctx, pool_id);
    // Add player team to pool first
    let pool_team_id = add_team_to_pool(ctx, pool_id, &player_team, pid)?;

    let enemy_team_id = if let Some(team_id) = pool_teams.choose(&mut ctx.rng()) {
        *team_id
    } else {
        0
    };
    create_battle(ctx, m, pid, pool_team_id, enemy_team_id)?;
    m.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_boss_battle(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let m_id = m.id;
    let floor = m.floor;

    // Load player team
    let player_team_id = m.team_load(ctx)?.id;
    let player_team = ctx.load::<NTeam>(player_team_id)?.load_all(ctx)?.take();

    // Ensure floor pool exists and add player's team to it
    let pool_id = ensure_floor_pool(ctx, floor)?;
    let pool_team_id = add_team_to_pool(ctx, pool_id, &player_team, pid)?;

    // Get boss team ID
    let enemy_team_id = get_floor_boss_team_id(ctx, floor)
        .or_else(|| get_floor_boss_team_id(ctx, floor - 1))
        .unwrap_or(0);

    // Create battle
    create_battle(ctx, m, pid, pool_team_id, enemy_team_id)?;

    m.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_complete(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    if m.active {
        return Err("Match is still active".into());
    } else {
        m.delete_recursive(ctx);
        Ok(())
    }
}

#[reducer]
fn match_change_action_range(
    ctx: &ReducerContext,
    slot_id: u64,
    start: u8,
    length: u8,
) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let mut slot = ctx.load::<NFusionSlot>(slot_id)?;
    if slot.owner != pid {
        return Err("Fusion slot not owned by player".to_string());
    }
    slot.set_actions(UnitActionRange { start, length });
    slot.save(ctx)?;
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let gs = ctx.global_settings();
    let pid = player.id;
    for mut m in NMatch::collect_owner(ctx, player.id) {
        m.load_all(ctx)?.delete_recursive(ctx);
    }
    let mut m = NMatch::new(
        ctx.next_id(),
        pid,
        gs.match_g.initial,
        0,
        3,
        true,
        false,
        default(),
    );
    let mut team = NTeam::new(ctx.next_id(), pid);
    for i in 0..gs.team_slots as i32 {
        let mut fusion = NFusion::new(ctx.next_id(), pid, default(), i, 0, 0, 0, 1);
        let slot = NFusionSlot::new(ctx.next_id(), pid, 0, default());
        fusion.slots_push(slot)?;
        team.fusions_push(fusion)?;
    }
    m.team_set(team)?;
    let mid = m.id;
    m.save(ctx)?;
    let mut m = ctx.load::<NMatch>(mid).track()?;
    m.fill_shop_case(ctx, false).track()?;
    player.active_match_set(m)?;
    debug!("{player:?}");
    player.save(ctx).track()?;
    Ok(())
}

impl NMatch {
    fn buy(&mut self, _ctx: &ServerContext, price: i32) -> NodeResult<()> {
        if self.g < price {
            return Err(NodeError::custom(format!(
                "Can't afford: price = {price} match g = {}",
                self.g
            )));
        }
        self.g_set(self.g - price)?;
        Ok(())
    }
    fn unlink_unit(&mut self, ctx: &ServerContext, unit_id: u64) -> Option<u64> {
        let links = TNodeLink::parents_of_kind(ctx.rctx(), unit_id, NodeKind::NFusionSlot, true);
        if links.len() > 1 {
            error!("Unit#{} linked to {} slots", unit_id, links.len());
        }
        let res = links.first().map(|l| l.parent);
        for link in links {
            ctx.rctx().db.node_links().delete(link);
        }
        res
    }
    fn fill_shop_case(&mut self, ctx: &ServerContext, units: bool) -> NodeResult<()> {
        let gs = ctx.global_settings();

        let unit_price = gs.match_g.unit_buy;
        let house_price = gs.match_g.house_buy;
        let owned_houses: HashSet<String> = HashSet::from_iter(
            self.team_load(ctx)
                .track()?
                .houses_load(ctx)
                .track()
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
            .flat_map(|h| {
                ctx.get_children_of_kind(h.id, NodeKind::NUnit)
                    .unwrap_or_default()
            })
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
        self.set_dirty(true);
        Ok(())
    }
}
