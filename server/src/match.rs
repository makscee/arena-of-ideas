use std::mem;

use schema::MatchState;
use spacetimedb::rand::{Rng, seq::SliceRandom};
use strum::IntoEnumIterator;

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
) -> NodeResult<u64> {
    let left_team = ctx.load::<NTeam>(player_team_id)?;
    let right_team = ctx.load::<NTeam>(enemy_team_id).unwrap_or_default();

    let battle_id =
        crate::battle_table::TBattle::create(ctx, player_id, &left_team, &right_team, default())?;

    m.pending_battle = Some(battle_id);
    Ok(battle_id)
}

#[reducer]
fn match_shop_buy(ctx: &ReducerContext, shop_idx: u8) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
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
    let card_kind = slot.card_kind;
    m.pay(ctx, price).track()?;
    let mid = m.id;
    m.take().save(ctx).track()?;
    let mut m = ctx.load::<NMatch>(mid)?;

    match card_kind {
        CardKind::Unit => {
            let mut team_unit = ctx
                .load::<NUnit>(node_id)
                .track()?
                .load_components(ctx)
                .track()?
                .take();
            let house_id = ctx.collect_kind_parents(team_unit.id, NodeKind::NHouse)?[0];
            team_unit =
                team_unit
                    .remap_ids(ctx)
                    .with_state(NUnitState::new(ctx.next_id(), pid, 1, 0));
            let id = team_unit.id;
            team_unit.save(ctx).track()?;
            id.add_parent(ctx.rctx(), mid)?;
            id.add_parent(ctx.rctx(), house_id)?;
        }
        CardKind::House => {
            let house = ctx
                .load::<NHouse>(node_id)
                .track()?
                .load_components(ctx)
                .track()?
                .take();
            let house_to_use = if let Some(existing_house) = m
                .shop_pool_load(ctx)
                .track()?
                .houses_load(ctx)
                .track()?
                .iter_mut()
                .find(|h| h.house_name == house.house_name)
            {
                let stax = existing_house.state().track()?.stax;
                existing_house.state_load(ctx).track()?.stax_set(stax + 1);
                existing_house.id
            } else {
                let new_house = house.remap_ids(ctx).with_owner(pid).with_state(NState::new(
                    ctx.next_id(),
                    pid,
                    1,
                ));
                let new_house_id = new_house.id;
                new_house.save(ctx).track()?;
                m.shop_pool_load(ctx)
                    .track()?
                    .id
                    .add_child(ctx.rctx(), new_house_id)
                    .track()?;
                new_house_id
            };
            let all_core_units = NUnit::collect_owner(ctx, ID_CORE);
            for _ in 0..5 {
                if let Some(mut unit) = all_core_units.choose(&mut ctx.rng()).cloned() {
                    let new_unit = unit
                        .load_components(ctx)
                        .track()?
                        .take()
                        .remap_ids(ctx)
                        .with_owner(pid);
                    let new_unit_id = new_unit.id;
                    new_unit.save(ctx).track()?;
                    new_unit_id.add_parent(ctx.rctx(), house_to_use)?;
                    new_unit_id.add_parent(ctx.rctx(), m.shop_pool()?.id)?;
                } else {
                    return Err("No core units available".to_string());
                }
            }
        }
    }
    if card_kind == CardKind::House {
        let mut m = ctx.load::<NMatch>(mid).track()?;
        m.fill_shop_case(ctx).track()?;
        m.save(ctx).track()?;
    }
    Ok(())
}

#[reducer]
fn match_move_unit(ctx: &ReducerContext, unit_id: u64, slot_index: i32) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;
    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".to_string());
    }
    let m = player.active_match_load(ctx)?;
    let mid = m.id;
    let prev_slot = m.unlink_unit(ctx, unit_id)?;
    if let Some(target_slot) = m
        .slots_load(ctx)?
        .iter_mut()
        .find(|x| x.index == slot_index)
    {
        if let Ok(unit_in_target) = target_slot.unit_load_id(ctx) {
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
    m.unlink_unit(ctx, unit_id)?;
    m.id.add_child(ctx.rctx(), unit_id)?;
    Ok(())
}

#[reducer]
fn match_shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.fill_shop_case(ctx)?;
    m.pay(ctx, ctx.global_settings().match_settings.reroll)?;
    player.save(ctx)?;
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
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let current_floor = m.floor;

    let Some(battle_id) = m.pending_battle else {
        return Err("No pending battle found".to_string());
    };
    debug!("Submit result: flr={current_floor} result={result} battle_id={battle_id}");
    if battle_id != id {
        return Err("Wrong Battle id".into());
    }

    // Update battle result in TBattle table
    crate::battle_table::TBattle::update_result(ctx.rctx(), battle_id, result)?;

    // Move pending battle to history
    m.battle_history.push(battle_id);
    m.pending_battle = None;

    let current_state = m.state;

    let mut arena = ctx.load::<NArena>(ID_ARENA)?;
    match current_state {
        MatchState::ChampionBattle => {
            if result {
                // Won champion battle - create new floor with boss and pool
                let player_team = m.build_team(ctx)?.load_all(ctx)?.take();

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
                let player_team = m.build_team(ctx)?.load_all(ctx)?.take();

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
                    m.fill_shop_case(ctx)?;
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
                m.fill_shop_case(ctx)?;
            } else {
                // Lost regular battle
                m.lives -= 1;

                if m.lives <= 0 {
                    m.active_set(false);
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
    let player_team = m.build_team(ctx)?.load_all(ctx)?.take();
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
    let mut player = ctx.player().track()?;
    let pid = player.id;
    let m = player.active_match_load(ctx).track()?;
    let floor = m.floor;
    let last_floor = ctx.load::<NArena>(ID_ARENA)?.last_floor;
    if m.floor > last_floor {
        m.set_state(MatchState::ChampionBattle);
        let player_team = m.build_team(ctx)?.load_all(ctx)?.take();
        let pool_id = ensure_floor_pool(ctx, floor).track()?;
        let player_team_id = add_team_to_pool(ctx, pool_id, &player_team, pid)?;
        let enemy_team_id = get_floor_boss_team_id(ctx, floor - 1).unwrap_or(0);
        create_battle(ctx, m, pid, player_team_id, enemy_team_id).track()?;
    } else {
        m.set_state(MatchState::BossBattle);
        let player_team = m.build_team(ctx)?.load_all(ctx)?.take();
        let pool_id = ensure_floor_pool(ctx, floor).track()?;
        let pool_team_id = add_team_to_pool(ctx, pool_id, &player_team, pid)?;
        let enemy_team_id = get_floor_boss_team_id(ctx, floor).unwrap_or(0);
        create_battle(ctx, m, pid, pool_team_id, enemy_team_id).track()?;
    }
    m.take().save(ctx).track()?;
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
fn match_start_fusion(ctx: &ReducerContext, source_id: u64, target_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
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

    let m = player.active_match_load(ctx)?;

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
    m.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_choose_fusion(ctx: &ReducerContext, fusion_index: i32) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let Some((source_id, target_id, ref variants)) = m.fusion.take() else {
        return Err("No fusion in progress".into());
    };
    m.take().save(ctx)?;
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
    merged_unit.clone().save(ctx)?;
    for house in houses {
        house.add_child(ctx.rctx(), unit_id).track()?;
    }
    slot.add_child(ctx.rctx(), unit_id).track()?;
    Ok(())
}

#[reducer]
fn match_cancel_fusion(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;

    let m = player.active_match_load(ctx)?;
    m.fusion = None;
    m.take().save(ctx)?;
    Ok(())
}

fn create_fused_unit(
    ctx: &mut ServerContext,
    source: &mut NUnit,
    target: &mut NUnit,
    fusion_type: FusionType,
) -> NodeResult<PackedNodes> {
    let mut new_unit = target.clone();
    let (desc_a, desc_b) = (
        &mut target.description()?.description.clone(),
        &mut source.description()?.description.clone(),
    );

    let mut front_name = target.unit_name.clone();
    let mut back_name = source.unit_name.clone();
    match fusion_type {
        FusionType::StickFront => {
            mem::swap(desc_a, desc_b);
            mem::swap(&mut front_name, &mut back_name);
        }
        FusionType::StickBack | FusionType::PushBack => {}
    }
    let front_half = front_name.len() / 2;
    let back_half = back_name.len() / 2;
    let (name_a, name_b) = (
        &mut (&front_name[..front_half]).to_owned(),
        &mut (&back_name[back_name.len() - back_half..]).to_owned(),
    );

    new_unit.unit_name = format!("{name_a}{name_b}");
    new_unit.description_load(ctx)?.description = format!("{desc_a}\n{desc_b}");
    let reactions = &mut new_unit.behavior_load(ctx)?.reactions;
    match fusion_type {
        FusionType::StickFront => {
            *reactions = source.behavior()?.reactions.clone();
            reactions.last_mut().unwrap().actions.extend(
                target
                    .behavior()?
                    .reactions
                    .first()
                    .unwrap()
                    .actions
                    .clone()
                    .into_iter(),
            );
            reactions.extend(target.behavior()?.reactions[1..].iter().cloned());
        }
        FusionType::StickBack => {
            reactions.last_mut().unwrap().actions.extend(
                source
                    .behavior()?
                    .reactions
                    .first()
                    .unwrap()
                    .actions
                    .clone()
                    .into_iter(),
            );
            reactions.extend(source.behavior()?.reactions[1..].iter().cloned());
        }
        FusionType::PushBack => {
            reactions.extend(source.behavior()?.reactions.clone().into_iter());
        }
    }

    let actions = &mut new_unit.representation_load(ctx)?.material.0;
    match fusion_type {
        FusionType::StickFront => {
            *actions = source.representation()?.material.0.clone();
            actions.push(PainterAction::paint);
            actions.extend(target.representation()?.material.0.clone().into_iter());
        }
        FusionType::StickBack | FusionType::PushBack => {
            actions.push(PainterAction::paint);
            actions.extend(source.representation()?.material.0.clone().into_iter());
        }
    }
    new_unit.stats_load(ctx)?.hp += source.stats()?.hp;
    new_unit.stats_load(ctx)?.pwr += source.stats()?.pwr;
    new_unit.state_load(ctx)?.stax += source.state()?.stax;
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
    let m = NMatch::new(
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
    let mid = m.id;
    m.save(ctx)?;
    let mut m = ctx.load::<NMatch>(mid).track()?;
    m.fill_shop_case(ctx).track()?;
    player.active_match_set(m)?;
    player.save(ctx).track()?;
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
        self.g_set(self.g - price);
        Ok(())
    }

    fn build_team(&mut self, ctx: &mut ServerContext) -> NodeResult<NTeam> {
        self.load_all(ctx)?;
        let mut team = NTeam::new(ctx.next_id(), self.owner);

        // Copy slots
        for slot in self.slots.iter() {
            team.slots_push(slot.clone())?;
        }

        // Copy houses but remove any references to bench or shop_pool units
        for mut house in self
            .shop_pool_load(ctx)?
            .houses
            .iter()
            .cloned()
            .collect_vec()
        {
            if let RefMultiple::Ids(unit_ids) = &mut house.units {
                // Keep only units that are in slots
                let slot_unit_ids: Vec<u64> = self
                    .slots
                    .iter()
                    .filter_map(|s| s.unit().ok().map(|u| u.id))
                    .collect();
                unit_ids.retain(|id| slot_unit_ids.contains(id));
            }
            team.houses_push(house)?;
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

        let shop_pool_units = self.shop_pool_load(ctx).track()?.units_load(ctx)?;
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

impl NMatch {
    fn unlink_unit(&mut self, ctx: &mut ServerContext, unit_id: u64) -> NodeResult<Option<u64>> {
        self.id.remove_child(ctx.rctx(), unit_id);
        for slot in self.slots_load(ctx)? {
            if slot.unit_load(ctx).is_ok_and(|u| u.id == unit_id) {
                slot.id.remove_child(ctx.rctx(), unit_id);
                return Ok(Some(slot.id));
            }
        }
        Ok(None)
    }
}
