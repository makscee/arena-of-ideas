use std::mem;

use schema::MatchState;
use spacetimedb::rand::{Rng, seq::SliceRandom};
use strum::IntoEnumIterator;

use crate::battle_table::TBattle;

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
        .and_then(|boss| boss.team.load_node(ctx).ok().map(|t| t.id))
}

fn ensure_floor_pool(ctx: &mut ServerContext, floor: i32) -> NodeResult<u64> {
    let mut arena = ctx.load::<NArena>(ID_ARENA)?.load_all(ctx)?.take();
    if arena.last_floor < floor {
        arena.last_floor += 1;
        let pool = NFloorPool::new(ctx.next_id(), ID_ARENA, floor);
        let pool_id = pool.id;
        let boss = NFloorBoss::new(ctx.next_id(), ID_ARENA, floor);
        arena.floor_pools.push(pool)?;
        arena.floor_bosses.push(boss)?;
        ctx.source_mut().commit(arena)?;
        Ok(pool_id)
    } else {
        arena
            .floor_pools
            .load_nodes(ctx)?
            .into_iter()
            .find_map(|f| if f.floor == floor { Some(f.id) } else { None })
            .to_custom_err_fn(|| format!("Pool floor {floor} not found"))
    }
}

fn add_team_to_pool(ctx: &mut ServerContext, pool_id: u64, team: NTeam) -> NodeResult<()> {
    let mut pool = ctx.load::<NFloorPool>(pool_id)?;
    pool.teams.load_mut(ctx)?.push(team)?;
    ctx.source_mut().commit(pool)?;
    Ok(())
}

fn create_battle(
    ctx: &mut ServerContext,
    m: &mut NMatch,
    player_id: u64,
    player_team_id: u64,
    enemy_team_id: u64,
) -> NodeResult<u64> {
    let left_team = ctx.load::<NTeam>(player_team_id)?;
    let right_team = ctx.load::<NTeam>(enemy_team_id).unwrap_or_default();

    let battle_id = TBattle::create(ctx, player_id, &left_team, &right_team, default())?;

    m.pending_battle = Some(battle_id);
    Ok(battle_id)
}

#[reducer]
fn match_shop_buy(ctx: &ReducerContext, shop_idx: u8) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let mut m = player.active_match.load_node(ctx)?.load_all(ctx)?.take();
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
    m.pay(ctx, price).track()?;

    match card_kind {
        CardKind::Unit => {
            let mut team_unit = ctx
                .load::<NUnit>(node_id)
                .track()?
                .load_components(ctx)
                .track()?
                .take();
            let house_id = ctx.first_parent(team_unit.id, NodeKind::NHouse)?;
            team_unit =
                team_unit
                    .remap_ids(ctx)
                    .with_state(NUnitState::new(ctx.next_id(), pid, 1, 0));
            let unit_id = team_unit.id;
            m.bench.push(team_unit)?;
        }
        CardKind::House => {
            let house = ctx
                .load::<NHouse>(node_id)
                .track()?
                .load_all(ctx)
                .track()?
                .take();
            let shop_pool = m.shop_pool.get_mut()?;
            let houses = shop_pool.houses.get_mut()?;
            let house_to_use = if let Some(existing_house) =
                houses.iter_mut().find(|h| h.house_name == house.house_name)
            {
                existing_house.state.get_mut()?.stax += 1;
                existing_house
            } else {
                let new_house = house.remap_ids(ctx).with_owner(pid).with_state(NState::new(
                    ctx.next_id(),
                    pid,
                    1,
                ));
                shop_pool.houses.push(new_house)?
            };
            let all_core_units = NUnit::collect_owner(ctx, ID_CORE);
            if all_core_units.is_empty() {
                return Err("No core units found".into());
            }
            for mut unit in all_core_units.choose_multiple(&mut ctx.rng(), 5).cloned() {
                let new_unit = unit
                    .load_components(ctx)
                    .track()?
                    .take()
                    .remap_ids(ctx)
                    .with_owner(pid);
                house_to_use.units.push_id(new_unit.id)?;
                shop_pool.units.push(new_unit)?;
            }
        }
    }
    if card_kind == CardKind::House {
        m.fill_shop_case(ctx).track()?;
    }
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn match_move_unit(ctx: &ReducerContext, unit_id: u64, slot_index: i32) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let mut m = player.active_match.load_node(ctx)?;
    let mid = m.id;
    let prev_slot = m.unlink_unit(ctx, unit_id)?;
    let mut slots = m.slots.load_nodes(ctx)?;
    if let Some(target_slot) = slots.iter_mut().find(|x| x.index == slot_index) {
        if let Ok(unit_in_target) = target_slot.unit.load_node(ctx).map(|u| u.id) {
            target_slot.id.remove_child(ctx.rctx(), unit_in_target);
            if let Some(prev_slot) = prev_slot {
                unit_in_target.add_parent(ctx.rctx(), prev_slot)?;
            } else {
                unit_in_target.add_parent(ctx.rctx(), mid)?;
            }
        }
        target_slot.id.add_child(ctx.rctx(), unit_id)?;
    } else {
        return Err("Invalid slot index".to_string());
    }
    Ok(())
}

#[reducer]
fn match_sell_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let mut m = player.active_match.load_node(ctx)?;
    m.g += ctx.global_settings().match_settings.unit_sell;
    unit.delete_recursive(ctx);
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn match_bench_unit(ctx: &ReducerContext, unit_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let mut m = player.active_match.load_node(ctx)?;
    m.unlink_unit(ctx, unit_id)?;
    m.id.add_child(ctx.rctx(), unit_id)?;
    Ok(())
}

#[reducer]
fn match_shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let mut m = player.active_match.load_node(ctx)?;
    m.fill_shop_case(ctx)?;
    m.pay(ctx, ctx.global_settings().match_settings.reroll)?;
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn match_submit_battle_result(
    ctx: &ReducerContext,
    id: u64,
    result: bool,
    _hash: u64,
) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let mut m = player.active_match.load_node(ctx)?;
    let current_floor = m.floor;

    let Some(battle_id) = m.pending_battle else {
        return Err("No pending battle found".to_string());
    };
    debug!("Submit result: flr={current_floor} result={result} battle_id={battle_id}");
    if battle_id != id {
        return Err("Wrong Battle id".into());
    }
    let battle = TBattle::load(ctx, battle_id)?;
    let player_team = PackedNodes::from_string(&battle.left_team)?.unpack::<NTeam>()?;
    battle.update_result(ctx, result)?;

    // Move pending battle to history
    m.battle_history.push(battle_id);
    m.pending_battle = None;

    let current_state = m.state;
    let mut arena = ctx.load::<NArena>(ID_ARENA)?;
    match current_state {
        MatchState::ChampionBattle => {
            if result {
                // Create new boss team for the new floor
                let new_boss_team = player_team.clone().remap_ids(ctx).with_owner(pid);
                let mut new_floor_boss = NFloorBoss::new(ctx.next_id(), ID_ARENA, current_floor);
                new_floor_boss.team = Owned::new_loaded(new_floor_boss.id, new_boss_team.clone());
                let mut new_floor_pool = NFloorPool::new(ctx.next_id(), ID_ARENA, current_floor);
                new_floor_pool
                    .teams
                    .push(new_boss_team.clone().remap_ids(ctx))?;
                arena.floor_pools.load_mut(ctx)?.push(new_floor_pool)?;
                arena.floor_bosses.load_mut(ctx)?.push(new_floor_boss)?;
                arena.last_floor = current_floor;
                ctx.source_mut().commit(arena)?;
            }
            m.active = false;
        }
        MatchState::BossBattle => {
            if result {
                // Create new boss team
                let new_boss_team = player_team.clone().remap_ids(ctx).with_owner(pid);
                let last_floor = arena.last_floor;
                let mut bosses = arena.floor_bosses.load_nodes(ctx)?;
                let mut boss = bosses
                    .iter_mut()
                    .find(|f| f.floor == current_floor)
                    .to_custom_err_fn(|| format!("Floor boss not found for {current_floor}"))?
                    .clone();
                boss.team.set_loaded(new_boss_team);
                ctx.source_mut().commit(boss)?;

                if current_floor == last_floor {
                    m.state = MatchState::ChampionShop;
                    m.floor += 1;
                    m.g += ctx.global_settings().match_settings.initial;
                    m.fill_shop_case(ctx)?;
                } else {
                    m.active = false;
                }
            } else {
                m.active = false;
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
                m.fill_shop_case(ctx)?;
            } else {
                // Lost regular battle
                m.lives -= 1;

                if m.lives <= 0 {
                    m.active = false;
                } else {
                    // Continue on same floor
                    m.state = MatchState::Shop;
                    m.g += ctx.global_settings().match_settings.initial;
                    m.fill_shop_case(ctx)?;
                }
            }
        }
        MatchState::Shop | MatchState::ChampionShop => {
            // This shouldn't happen - no battle should be pending in shop state
            return Err("Invalid state: battle result submitted while in shop".into());
        }
    }
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let mut m = player.active_match.load_node(ctx)?;
    let floor = m.floor;

    let last_floor = ctx.load::<NArena>(ID_ARENA)?.last_floor;
    if floor >= last_floor {
        return Err(
            "Regular battles not allowed on the last floor. Must fight the boss!".to_string(),
        );
    }

    m.state = MatchState::RegularBattle;
    let player_team = m.build_team(ctx)?;
    let player_team_id = player_team.id;
    let pool_id = ensure_floor_pool(ctx, floor)?;
    let pool_teams = get_floor_pool_teams(ctx, pool_id);
    add_team_to_pool(ctx, pool_id, player_team)?;

    let enemy_team_id = if let Some(team_id) = pool_teams.choose(&mut ctx.rng()) {
        *team_id
    } else {
        0
    };
    create_battle(ctx, &mut m, pid, player_team_id, enemy_team_id)?;
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn match_boss_battle(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;
    let mut m = player.active_match.load_node(ctx)?;
    let floor = m.floor;
    let last_floor = ctx.load::<NArena>(ID_ARENA)?.last_floor;
    let enemy_team_id = if m.floor > last_floor {
        m.state = MatchState::ChampionBattle;
        get_floor_boss_team_id(ctx, floor - 1).unwrap_or(0)
    } else {
        m.state = MatchState::BossBattle;
        get_floor_boss_team_id(ctx, floor).unwrap_or(0)
    };
    let player_team = m.build_team(ctx)?;
    let player_team_id = player_team.id;
    let pool_id = ensure_floor_pool(ctx, floor).track()?;
    add_team_to_pool(ctx, pool_id, player_team)?;
    create_battle(ctx, &mut m, pid, player_team_id, enemy_team_id).track()?;
    ctx.source_mut().commit(m).track()?;
    Ok(())
}

#[reducer]
fn match_complete(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let mut m = player.active_match.load_node(ctx)?;
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
    let player = ctx.player()?;
    let mut m = player.active_match.load_node(ctx)?;
    m.load_all(ctx)?.delete_recursive(ctx);
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
    if !unit.check_stackable(&mut target_unit, ctx)? {
        return Err(NodeError::custom(format!(
            "Units {} and {} cannot be stacked",
            unit.id, target_unit.id
        ))
        .into());
    }

    let unit_state = unit.state.load_node(ctx)?;
    let stax = unit_state.stax;
    unit.delete_recursive(ctx);

    let mut target_unit_state = target_unit.state.load_node(ctx)?;
    target_unit_state.stax += stax;
    ctx.source_mut().commit(target_unit_state)?;

    let mut target_behavior = target_unit.behavior.load_node(ctx)?;
    let mut target_unit_stats = target_behavior.stats.load_node(ctx)?;
    target_unit_stats.hp += stax;
    target_unit_stats.pwr += stax;
    ctx.source_mut().commit(target_unit_stats)?;

    Ok(())
}

#[reducer]
fn match_start_fusion(ctx: &ReducerContext, source_id: u64, target_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let pid = player.id;

    let mut source = ctx.load::<NUnit>(source_id)?.load_components(ctx)?.take();
    let mut target = ctx.load::<NUnit>(target_id)?.load_components(ctx)?.take();

    if source.owner != pid || target.owner != pid {
        return Err("Unit not owned by player".into());
    }
    if !source.check_fusible(ctx)? {
        return Err(NodeError::custom(format!("Unit {} is not fusible", source.id)).into());
    }
    if !target.check_fusible(ctx)? {
        return Err(NodeError::custom(format!("Unit {} is not fusible", target.id)).into());
    }
    if !source.check_fusible_with(&mut target, ctx)? {
        return Err(NodeError::custom(format!(
            "Units {} and {} cannot be fused together",
            source.id, target.id
        ))
        .into());
    }

    let mut m = player.active_match.load_node(ctx)?;

    let mut packed_variants = Vec::new();
    for fusion_type in FusionType::iter() {
        packed_variants.push(create_fused_unit(
            ctx,
            &mut source,
            &mut target,
            fusion_type,
        )?);
    }

    m.fusion = Some((source_id, target_id, packed_variants));
    ctx.source_mut().commit(m)?;
    Ok(())
}

#[reducer]
fn match_choose_fusion(ctx: &ReducerContext, fusion_index: i32) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;
    let mut m = player.active_match.load_node(ctx)?;
    let Some((source_id, target_id, ref variants)) = m.fusion.take() else {
        return Err("No fusion in progress".into());
    };
    ctx.source_mut().commit(m)?;
    let fusion_idx = fusion_index as usize;
    if fusion_idx >= variants.len() {
        return Err("Invalid fusion type index".into());
    }
    let houses = target_id
        .collect_kind_parents(ctx.rctx(), NodeKind::NHouse)
        .into_iter()
        .chain(source_id.collect_kind_parents(ctx.rctx(), NodeKind::NHouse))
        .collect_vec();
    let slot = target_id
        .find_kind_parent(ctx.rctx(), NodeKind::NTeamSlot)
        .to_not_found()?;
    TNode::delete_by_id_recursive(ctx.rctx(), target_id);
    TNode::delete_by_id_recursive(ctx.rctx(), source_id);

    let packed = &variants[fusion_idx];
    let merged_unit = NUnit::unpack(packed)?.remap_ids(ctx).with_owner(player.id);
    let unit_id = merged_unit.id;
    ctx.source_mut().commit(merged_unit)?;
    for house in houses {
        house.add_child(ctx.rctx(), unit_id).track()?;
    }
    slot.add_child(ctx.rctx(), unit_id).track()?;
    Ok(())
}

#[reducer]
fn match_cancel_fusion(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let player = ctx.player()?;

    let mut m = player.active_match.load_node(ctx)?;
    m.fusion = None;
    ctx.source_mut().commit(m)?;
    Ok(())
}

fn create_fused_unit(
    ctx: &mut ServerContext,
    source: &mut NUnit,
    target: &mut NUnit,
    fusion_type: FusionType,
) -> NodeResult<PackedNodes> {
    let mut new_unit = target.clone();

    let mut front_name = target.unit_name.clone();
    let mut back_name = source.unit_name.clone();
    match fusion_type {
        FusionType::StickFront => {
            mem::swap(&mut front_name, &mut back_name);
        }
        FusionType::StickBack | FusionType::PushBack => {}
    }
    let front_half = front_name.len() / 2;
    let back_half = back_name.len() / 2;
    let name_a = &front_name[..front_half];
    let name_b = &back_name[back_name.len() - back_half..];

    new_unit.unit_name = format!("{name_a}{name_b}");

    let mut new_behavior = new_unit.behavior.load_node(ctx)?;
    let source_behavior = source.behavior.load_node(ctx)?;
    let target_behavior = target.behavior.load_node(ctx)?;
    let reactions = &mut new_behavior.reactions;
    match fusion_type {
        FusionType::StickFront => {
            *reactions = source_behavior.reactions.clone();
            reactions.last_mut().unwrap().effect.actions.extend(
                target_behavior
                    .reactions
                    .first()
                    .unwrap()
                    .effect
                    .actions
                    .clone()
                    .into_iter(),
            );
            reactions.extend(target_behavior.reactions[1..].iter().cloned());
        }
        FusionType::StickBack => {
            reactions.last_mut().unwrap().effect.actions.extend(
                source_behavior
                    .reactions
                    .first()
                    .unwrap()
                    .effect
                    .actions
                    .clone()
                    .into_iter(),
            );
            reactions.extend(source_behavior.reactions[1..].iter().cloned());
        }
        FusionType::PushBack => {
            reactions.extend(source_behavior.reactions.clone().into_iter());
        }
    }
    ctx.source_mut().commit(new_behavior)?;

    let new_behavior = new_unit.behavior.load_node(ctx)?;
    let mut new_representation = new_behavior.representation.load_node(ctx)?;
    let source_behavior = source.behavior.load_node(ctx)?;
    let source_representation = source_behavior.representation.load_node(ctx)?;
    let target_behavior = target.behavior.load_node(ctx)?;
    let target_representation = target_behavior.representation.load_node(ctx)?;
    let actions = &mut new_representation.material.0;
    match fusion_type {
        FusionType::StickFront => {
            *actions = source_representation.material.0.clone();
            actions.push(PainterAction::paint);
            actions.extend(target_representation.material.0.clone().into_iter());
        }
        FusionType::StickBack | FusionType::PushBack => {
            actions.push(PainterAction::paint);
            actions.extend(source_representation.material.0.clone().into_iter());
        }
    }
    ctx.source_mut().commit(new_representation)?;

    let mut new_behavior = new_unit.behavior.load_node(ctx)?;
    let mut new_stats = new_behavior.stats.load_node(ctx)?;
    let source_behavior = source.behavior.load_node(ctx)?;
    let source_stats = source_behavior.stats.load_node(ctx)?;
    new_stats.hp += source_stats.hp;
    new_stats.pwr += source_stats.pwr;
    ctx.source_mut().commit(new_stats)?;

    let mut new_state = new_unit.state.load_node(ctx)?;
    let source_state = source.state.load_node(ctx)?;
    new_state.stax += source_state.stax;
    ctx.source_mut().commit(new_state)?;

    Ok(new_unit.pack())
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
    let mut team_slots = Vec::new();
    for i in 0..gs.team_slots {
        team_slots.push(NTeamSlot::new(ctx.next_id(), pid, i as i32));
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
        vec![], // battle_history
        None,   // pending_battle
        None,   // fusion
    )
    .with_shop_pool(NShopPool::new(ctx.next_id(), pid))
    .with_slots(team_slots);
    m.fill_shop_case(ctx).track()?;
    player.active_match.set_loaded(m);
    ctx.source_mut().commit(player).track()?;
    Ok(())
}

impl NMatch {
    fn pay(&mut self, _ctx: &ServerContext, price: i32) -> NodeResult<()> {
        if self.g < price {
            return Err(NodeError::custom(format!(
                "Can't afford: price = {price} match g = {}",
                self.g
            )));
        }
        self.g -= price;
        Ok(())
    }

    fn build_team(&mut self, ctx: &mut ServerContext) -> NodeResult<NTeam> {
        self.load_all(ctx)?;
        let mut team = NTeam::new(ctx.next_id(), self.owner);

        // Copy slots
        let slots = self.slots.get_mut()?;
        let slot_units = slots
            .iter()
            .filter_map(|s| s.unit.get().ok().map(|u| u.id))
            .collect_vec();
        if slot_units.is_empty() {
            return Err(NodeError::custom("No units in slots"));
        }
        for slot in slots.iter() {
            team.slots.push(slot.clone())?;
        }

        // Copy houses but remove any references to bench or shop_pool units
        let shop_pool = self.shop_pool.get()?;
        for mut house in shop_pool.houses.get()?.clone() {
            house.units = RefMultiple::Ids {
                parent_id: house.id,
                node_ids: house
                    .units
                    .ids()?
                    .into_iter()
                    .filter(|u| slot_units.contains(u))
                    .collect_vec(),
            };
            team.houses.push(house)?;
        }

        Ok(team.remap_ids(ctx))
    }

    fn fill_shop_case(&mut self, ctx: &ServerContext) -> NodeResult<()> {
        let gs = ctx.global_settings();

        let unit_price = gs.match_settings.unit_buy;
        let house_price = gs.match_settings.house_buy;
        let house_chance = gs.match_settings.house_chance;

        let all_houses = NHouse::collect_owner(ctx, ID_CORE);
        let all_house_ids = all_houses.iter().map(|h| h.id).collect_vec();

        let shop_pool = self.shop_pool.load_node(ctx).track()?;
        let shop_pool_units = shop_pool.units.load_nodes(ctx).track()?;
        let has_units = !shop_pool_units.is_empty();

        let shop_case = (0..4)
            .map(|_| {
                let show_house = !has_units || ctx.rng().gen_bool(house_chance as f64 / 100.0);
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
                        node_id: shop_pool_units.choose(&mut ctx.rng()).unwrap().id,
                        sold: false,
                        price: unit_price,
                        buy_text: None,
                    }
                }
            })
            .collect_vec();
        self.shop_offers = vec![ShopOffer {
            buy_limit: None,
            case: shop_case,
        }];
        Ok(())
    }
}

impl NMatch {
    fn unlink_unit(&mut self, ctx: &mut ServerContext, unit_id: u64) -> NodeResult<Option<u64>> {
        self.id.remove_child(ctx.rctx(), unit_id);
        let slots = self.slots.load_nodes(ctx)?;
        for slot in slots.iter() {
            if slot.unit.load_node(ctx).is_ok_and(|u| u.id == unit_id) {
                slot.id.remove_child(ctx.rctx(), unit_id);
                return Ok(Some(slot.id));
            }
        }
        Ok(None)
    }
}
