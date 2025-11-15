use schema::MatchState;
use spacetimedb::rand::{Rng, seq::SliceRandom};

use super::*;

fn get_floor_bosses(ctx: &ServerContext) -> Vec<TNode> {
    TNode::collect_kind_owner(ctx.rctx(), NodeKind::NFloorBoss, ID_ARENA)
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

fn ensure_floor_pool(ctx: &mut ServerContext, floor: i32) -> NodeResult<u64> {
    let mut arena = ctx.load::<NArena>(ID_ARENA)?;
    if arena.last_floor < floor {
        arena.floor_pools_load(ctx)?;
        arena.floor_bosses_load(ctx)?;
        arena.last_floor += 1;
        let pool = NFloorPool::new(ctx.next_id(), ID_ARENA, floor);
        let pool_id = pool.id;
        let boss = NFloorBoss::new(ctx.next_id(), ID_ARENA, floor);
        arena.floor_pools_push(pool)?;
        arena.floor_bosses_push(boss)?;
        arena.save(ctx)?;
        Ok(pool_id)
    } else {
        arena
            .floor_pools_load(ctx)?
            .into_iter()
            .find_map(|f| if f.floor == floor { Some(f.id) } else { None })
            .to_custom_err_fn(|| format!("Pool floor {floor} not found"))
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
            unit.state_set(NState::new(ctx.next_id(), pid, 1))?;
            let unit_id = unit.id;
            unit.save(ctx)?;
            unit_id.add_parent(ctx.rctx(), house.id)?;
        }
        CardKind::House => {
            let house = ctx.load::<NHouse>(node_id)?.load_components(ctx)?.take();
            let team_houses = m.team()?.houses()?.clone();
            if let Some(existing_house) = team_houses
                .iter()
                .find(|h| h.house_name == house.house_name)
            {
                // Stack existing house
                let existing_house_id = existing_house.id;
                let mut existing_house = ctx.load::<NHouse>(existing_house_id)?;
                let existing_house_state = existing_house.state_load(ctx)?;
                existing_house_state.stax_set(existing_house_state.stax + 1);
                existing_house_state.clone().save(ctx)?;
            } else {
                // Add new house with initial state
                let house = house.remap_ids(ctx).with_owner(pid).with_state(NState::new(
                    ctx.next_id(),
                    pid,
                    1,
                ));
                m.team_mut()?.houses_push(house)?;
            }
        }
    }
    m.buy(ctx, price).track()?;
    let mid = m.id;
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
    let fusion = slot
        .id
        .get_kind_parent(ctx.rctx(), NodeKind::NFusion)
        .to_not_found()?;

    let mut fusion = ctx.load::<NFusion>(fusion)?;
    let slots = fusion.slots_load(ctx)?;

    for slot in slots.iter_mut() {
        if let Ok(slot_unit) = slot.unit_load(ctx) {
            if slot_unit.unit_name == unit.unit_name && slot_unit.id != unit_id {
                return Err("Cannot place duplicate units in the same fusion".into());
            }
        }
    }

    let other_units_exist = slots
        .iter_mut()
        .any(|s| s.unit_load_id(ctx).is_ok_and(|id| id != unit_id));

    let total_allocated: u8 = slots
        .iter_mut()
        .filter_map(|s| {
            if s.unit_load_id(ctx).is_ok_and(|id| id != unit_id) {
                Some(s.actions.length)
            } else {
                None
            }
        })
        .sum();

    let available = (fusion.actions_limit as u8).saturating_sub(total_allocated);
    slot.unit = Ref::Id(unit.id);
    slot.set_actions(UnitActionRange {
        start: 0,
        length: available,
    });
    slot.save(ctx)?;

    if !other_units_exist {
        fusion.trigger_unit = Ref::Id(unit_id);
        fusion.set_dirty(true);
    }

    apply_slots_limit(ctx, &mut fusion).track()?;
    fusion.save(ctx).track()?;

    if let Some(old_slot_id) = old_target_id {
        let old_slot = ctx.load::<NFusionSlot>(old_slot_id)?;
        let fusion_id = old_slot
            .id
            .get_kind_parent(ctx.rctx(), NodeKind::NFusion)
            .to_not_found()?;
        let mut old_fusion = ctx.load::<NFusion>(fusion_id)?.load_all(ctx)?.take();
        if old_fusion.trigger_unit.id() == Some(unit_id) {
            let new_trigger = old_fusion
                .slots()?
                .iter()
                .filter_map(|s| s.unit.id())
                .find(|&id| id != unit_id)
                .map(|id| Ref::Id(id))
                .unwrap_or(Ref::None);
            old_fusion.trigger_unit = new_trigger;
            old_fusion.set_dirty(true);
            old_fusion.save(ctx)?;
        }
    }

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
    m.g_set(ctx.global_settings().match_settings.unit_sell + m.g);
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
    fusion.actions_limit += 1;
    let slots = fusion.slots_load(ctx)?;
    let price = ctx.global_settings().match_settings.fusion_slot_mul * slots.len() as i32;
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
    m.buy(ctx, ctx.global_settings().match_settings.reroll)?;
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

    let mut battles = m.battles_load(ctx)?;
    battles.sort_by_key(|b| b.id);
    let battle = battles.last_mut().unwrap();
    debug!("Submit result: flr={current_floor} result={result} {battle:?}");
    if battle.id != id {
        return Err("Wrong Battle id".into());
    }
    if battle.result.is_some() {
        return Err("Battle result already submitted".into());
    }
    battle.set_result(Some(result));
    battle.set_hash(hash);
    battle.id.add_parent(ctx.rctx(), ID_ARENA)?;
    battle.take().save(ctx)?;

    let current_state = m.state;

    let mut arena = ctx.load::<NArena>(ID_ARENA)?;
    match current_state {
        MatchState::ChampionBattle => {
            if result {
                // Won champion battle - create new floor with boss and pool
                let player_team_id = m.team_load(ctx)?.id;
                let player_team = ctx.load::<NTeam>(player_team_id)?.load_all(ctx)?.take();

                // Create new boss team for the new floor
                let new_boss_team = player_team.clone().remap_ids(ctx).with_owner(pid);
                let mut new_floor_boss = NFloorBoss::new(ctx.next_id(), ID_ARENA, current_floor);
                new_floor_boss.team_set(new_boss_team.clone())?;
                let mut new_floor_pool = NFloorPool::new(ctx.next_id(), ID_ARENA, current_floor);
                new_floor_pool.teams_push(new_boss_team.clone().remap_ids(ctx))?;

                arena.floor_pools_load(ctx)?;
                arena.floor_bosses_load(ctx)?;
                arena.set_last_floor(current_floor);
                arena.floor_bosses_push(new_floor_boss)?;
                arena.floor_pools_push(new_floor_pool)?;
                arena.save(ctx)?;
            }
            m.active_set(false);
        }
        MatchState::BossBattle => {
            if result {
                // Won against boss - replace boss with player's team
                let player_team_id = m.team_load(ctx)?.id;
                let player_team = ctx.load::<NTeam>(player_team_id)?.load_all(ctx)?.take();

                // Create new boss team
                let new_boss_team = player_team.clone().remap_ids(ctx).with_owner(pid);
                let last_floor = arena.last_floor;
                let mut boss = arena
                    .floor_bosses_load(ctx)?
                    .iter()
                    .find(|f| f.floor == current_floor)
                    .to_custom_err_fn(|| format!("Floor boss not found for {current_floor}"))?
                    .clone();
                boss.team_set(new_boss_team)?;
                boss.save(ctx)?;
                if current_floor == last_floor {
                    m.set_state(MatchState::ChampionShop);
                    m.floor += 1;
                    m.g += ctx.global_settings().match_settings.initial;
                    m.fill_shop_case(ctx, false)?;
                } else {
                    m.active_set(false);
                }
            } else {
                m.active_set(false);
            }
        }
        MatchState::RegularBattle => {
            if result {
                // Won regular battle
                m.floor += 1;

                // Gain life every 5 floors
                if m.floor % 5 == 0 {
                    m.lives += 1;
                }

                m.state = MatchState::Shop;
                m.g += ctx.global_settings().match_settings.initial;
                m.fill_shop_case(ctx, false)?;
            } else {
                // Lost regular battle
                m.lives -= 1;

                if m.lives <= 0 {
                    m.active_set(false);
                } else {
                    // Continue on same floor
                    m.state = MatchState::Shop;
                    m.g += ctx.global_settings().match_settings.initial;
                    m.fill_shop_case(ctx, false)?;
                }
            }
        }
        MatchState::Shop | MatchState::ChampionShop => {
            // This shouldn't happen - no battle should be pending in shop state
            return Err("Invalid state: battle result submitted while in shop".into());
        }
    }
    m.set_dirty(true);
    player.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let floor = m.floor;

    let last_floor = ctx.load::<NArena>(ID_ARENA)?.last_floor;
    if floor >= last_floor {
        return Err(
            "Regular battles not allowed on the last floor. Must fight the boss!".to_string(),
        );
    }

    m.set_state(MatchState::RegularBattle);
    let player_team = m.team_load(ctx)?.load_all(ctx)?;
    let pool_id = ensure_floor_pool(ctx, floor)?;
    let pool_teams = get_floor_pool_teams(ctx, pool_id);
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
    let floor = m.floor;
    let last_floor = ctx.load::<NArena>(ID_ARENA)?.last_floor;
    if m.floor > last_floor {
        m.set_state(MatchState::ChampionBattle);
        let player_team_id = m.team_load(ctx)?.id;
        let enemy_team_id = get_floor_boss_team_id(ctx, floor - 1).unwrap_or(0);
        create_battle(ctx, m, pid, player_team_id, enemy_team_id)?;
    } else {
        m.set_state(MatchState::BossBattle);
        let player_team = m.team_load(ctx)?.load_all(ctx)?;
        let pool_id = ensure_floor_pool(ctx, floor)?;
        let pool_team_id = add_team_to_pool(ctx, pool_id, &player_team, pid)?;
        let enemy_team_id = get_floor_boss_team_id(ctx, floor).unwrap_or(0);
        create_battle(ctx, m, pid, pool_team_id, enemy_team_id)?;
    }
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
        m.load_all(ctx)?.delete_recursive(ctx);
        Ok(())
    }
}

#[reducer]
fn match_abandon(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.load_all(ctx)?.delete_recursive(ctx);
    Ok(())
}

fn apply_slots_limit(ctx: &mut ServerContext, fusion: &mut NFusion) -> NodeResult<()> {
    fusion.set_dirty(true);
    let limit = fusion.actions_limit;
    let mut used: i32 = 0;
    for slot in fusion
        .slots_load(ctx)?
        .into_iter()
        .sorted_by_key(|s| s.index)
    {
        if let Ok(mut unit) = slot.unit_load(ctx) {
            let b = unit.description_load(ctx)?.behavior_load(ctx)?;
            let actions = b.reaction.actions.len();
            slot.actions.start = slot.actions.start.min(actions as u8);
            slot.actions.length = slot
                .actions
                .length
                .min((actions - slot.actions.start as usize) as u8)
                .min((limit - used) as u8);
            used += slot.actions.length as i32;
            slot.set_dirty(true);
        } else {
            slot.set_actions(default());
        }
    }
    Ok(())
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
    let mut fusion = slot.load_parent::<NFusion>(ctx)?;
    slot.save(ctx)?;
    apply_slots_limit(ctx, &mut fusion)?;
    fusion.save(ctx)?;
    Ok(())
}

#[reducer]
fn match_change_trigger(
    ctx: &ReducerContext,
    fusion_id: u64,
    trigger_unit: u64,
) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let mut fusion = ctx.load::<NFusion>(fusion_id)?;
    if fusion.owner != pid {
        return Err("Fusion not owned by player".to_string());
    }
    ctx.load::<NUnit>(trigger_unit)?;

    // Verify trigger unit is in this fusion
    if trigger_unit != 0 {
        let slots = fusion.load_all(ctx)?.slots()?;
        let trigger_valid = slots.iter().any(|slot| {
            slot.unit
                .id()
                .map_or(false, |unit_id| unit_id == trigger_unit)
        });
        if !trigger_valid {
            return Err("Trigger unit must be in the fusion".to_string());
        }
    }
    fusion.trigger_unit = Ref::Id(trigger_unit);
    fusion.set_dirty(true);
    fusion.save(ctx)?;
    Ok(())
}

#[reducer]
fn match_stack_unit(ctx: &ReducerContext, unit_id: u64, target_unit_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;

    let mut unit = ctx.load::<NUnit>(unit_id)?;
    let mut target_unit = ctx.load::<NUnit>(target_unit_id)?;

    if unit.owner != pid || target_unit.owner != pid {
        return Err("Units not owned by player".into());
    }

    if unit.unit_name != target_unit.unit_name {
        return Err("Units must have same name to stack".into());
    }
    let stax = unit.state_load(ctx)?.stax;
    unit.delete_recursive(ctx);

    let target_unit_state = target_unit.state_load(ctx)?;
    target_unit_state.stax_set(target_unit_state.stax + stax);

    let target_unit_stats = target_unit.stats_load(ctx)?;
    target_unit_stats.hp_set(target_unit_stats.hp + stax);
    target_unit_stats.pwr_set(target_unit_stats.pwr + stax);

    target_unit.save(ctx)?;

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
        gs.match_settings.initial,
        1,
        3,
        true,
        MatchState::Shop,
        default(),
    );
    let mut team = NTeam::new(ctx.next_id(), pid);
    for i in 0..gs.team_slots as i32 {
        let mut fusion = NFusion::new(ctx.next_id(), pid, i, 0, 0, 0, 3);
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
        self.g_set(self.g - price);
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

        let unit_price = gs.match_settings.unit_buy;
        let house_price = gs.match_settings.house_buy;
        let house_chance = gs.match_settings.house_chance;

        let owned_houses: HashSet<String> = HashSet::from_iter(
            self.team_load(ctx)
                .track()?
                .houses_load(ctx)
                .track()
                .map(|h| h.into_iter().map(|h| h.house_name.clone()).collect_vec())
                .unwrap_or_default(),
        );
        let all_houses = NHouse::collect_owner(ctx, ID_CORE);
        let all_house_ids = all_houses.iter().map(|h| h.id).collect_vec();
        let units_from_owned_houses = all_houses
            .into_iter()
            .filter(|h| owned_houses.contains(&h.house_name))
            .flat_map(|h| {
                ctx.get_children_of_kind(h.id, NodeKind::NUnit)
                    .unwrap_or_default()
            })
            .collect_vec();

        let has_units = units && !units_from_owned_houses.is_empty();
        let has_houses = !all_house_ids.is_empty();

        let shop_case = (0..4)
            .map(|_| {
                if !has_units {
                    if has_houses {
                        ShopSlot {
                            card_kind: CardKind::House,
                            node_id: *all_house_ids.choose(&mut ctx.rng()).unwrap(),
                            sold: false,
                            price: house_price,
                            buy_text: None,
                        }
                    } else {
                        ShopSlot {
                            card_kind: CardKind::Unit,
                            node_id: *units_from_owned_houses.choose(&mut ctx.rng()).unwrap(),
                            sold: false,
                            price: unit_price,
                            buy_text: None,
                        }
                    }
                } else if !has_houses {
                    ShopSlot {
                        card_kind: CardKind::Unit,
                        node_id: *units_from_owned_houses.choose(&mut ctx.rng()).unwrap(),
                        sold: false,
                        price: unit_price,
                        buy_text: None,
                    }
                } else {
                    let show_house = ctx.rng().gen_bool(house_chance as f64 / 100.0);
                    if show_house {
                        ShopSlot {
                            card_kind: CardKind::House,
                            node_id: *all_house_ids.choose(&mut ctx.rng()).unwrap(),
                            sold: false,
                            price: house_price,
                            buy_text: None,
                        }
                    } else {
                        ShopSlot {
                            card_kind: CardKind::Unit,
                            node_id: *units_from_owned_houses.choose(&mut ctx.rng()).unwrap(),
                            sold: false,
                            price: unit_price,
                            buy_text: None,
                        }
                    }
                }
            })
            .collect_vec();
        self.set_shop_offers(
            [ShopOffer {
                buy_limit: None,
                case: shop_case,
            }]
            .into(),
        );
        Ok(())
    }
}
