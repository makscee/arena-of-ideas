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
            unit.state_set(NUnitState::new(ctx.next_id(), pid, 1, 0))?;
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
fn match_move_unit(ctx: &ReducerContext, unit_id: u64, slot_index: i32) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;

    let unit = ctx.load::<NUnit>(unit_id)?;
    if unit.owner != pid {
        return Err("Unit not owned by player".into());
    }

    let mut m = player.active_match_load(ctx)?;
    let mut team = m.team_load(ctx)?;

    // Find and clear existing slot if unit is already placed
    let mut slots = team.slots_load(ctx)?;
    for slot in slots.iter_mut() {
        if slot.unit_load_id(ctx).ok() == Some(unit_id) {
            slot.unit = Owned::None;
            slot.clone().save(ctx)?;
            break;
        }
    }

    // Place unit in new slot
    let mut slots = team.slots_load(ctx)?;
    if let Some(mut target_slot) = slots.iter_mut().find(|s| s.index == slot_index) {
        target_slot.unit = Owned::new_id(unit_id);
        target_slot.clone().save(ctx)?;
    } else {
        return Err("Slot not found".into());
    }

    team.clone().save(ctx)?;
    m.take().save(ctx)?;
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
    let mut m = player.active_match_load(ctx)?;
    let mut team = m.team_load(ctx)?;

    // Remove unit from slots and add to bench
    let mut slots = team.slots_load(ctx)?;
    for slot in slots.iter_mut() {
        if slot.unit_load_id(ctx).ok() == Some(unit_id) {
            slot.unit = Owned::None;
            slot.clone().save(ctx)?;
            break;
        }
    }

    team.benched_push(unit)?;
    team.clone().save(ctx)?;
    m.take().save(ctx)?;
    Ok(())
}

// match_buy_fusion_slot removed - no longer needed with new fusion system

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
    let mut player = ctx.player().track()?;
    let pid = player.id;
    let m = player.active_match_load(ctx).track()?;
    let floor = m.floor;
    let last_floor = ctx.load::<NArena>(ID_ARENA)?.last_floor;
    if m.floor > last_floor {
        m.set_state(MatchState::ChampionBattle);
        let player_team_id = m.team_load(ctx).track()?.id;
        let enemy_team_id = get_floor_boss_team_id(ctx, floor - 1).unwrap_or(0);
        create_battle(ctx, m, pid, player_team_id, enemy_team_id).track()?;
    } else {
        m.set_state(MatchState::BossBattle);
        let player_team = m.team_load(ctx)?.load_all(ctx)?;
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

// apply_slots_limit removed - no longer needed with new fusion system

// match_change_action_range and match_change_trigger removed - no longer needed with new fusion system

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
fn match_start_fusion(ctx: &ReducerContext, source_id: u64, target_id: u64) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;

    let mut source = ctx.load::<NUnit>(source_id)?.load_components(ctx)?.take();
    let mut target = ctx.load::<NUnit>(target_id)?.load_components(ctx)?.take();

    if source.owner != pid || target.owner != pid {
        return Err("Units not owned by player".into());
    }

    let source_state_stax = source.state_load(ctx)?.stax;
    let source_state_houses = source.house_ids(ctx);
    let source_action_count: usize = source
        .behavior_load(ctx)?
        .reactions
        .iter()
        .map(|r| r.actions.len())
        .sum();

    let target_state_stax = target.state_load(ctx)?.stax;
    let target_state_houses = target.house_ids(ctx);
    let target_action_count: usize = target
        .behavior_load(ctx)?
        .reactions
        .iter()
        .map(|r| r.actions.len())
        .sum();

    let source_can_fuse = if source_state_houses.is_empty() {
        source_state_stax >= 2
    } else {
        source_state_stax >= source_action_count as i32
    };

    let target_can_fuse = if target_state_houses.is_empty() {
        target_state_stax >= 2
    } else {
        target_state_stax >= target_action_count as i32
    };

    if !source_can_fuse || !target_can_fuse {
        return Err("Units do not meet fusion requirements".into());
    }

    let source_house = if source_state_houses.is_empty() {
        source.id
    } else {
        *source_state_houses.first().unwrap()
    };

    let target_house = if target_state_houses.is_empty() {
        target.id
    } else {
        *target_state_houses.first().unwrap()
    };

    if source_house == target_house {
        return Err("Units must be from different houses".into());
    }

    let mut m = player.active_match_load(ctx)?;

    let mut packed_variants = Vec::new();
    for fusion_type in [0i32, 1i32, 2i32].iter() {
        let merged_unit = create_fused_unit(ctx, pid, &mut source, &mut target, *fusion_type)?;
        let packed = merged_unit.pack();
        packed_variants.push(packed);
    }

    m.fusion = Some((source_id, target_id, packed_variants));
    m.take().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_choose_fusion(ctx: &ReducerContext, fusion_index: i32) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;
    let pid = player.id;

    let mut m = player.active_match_load(ctx)?;

    if let Some((source_id, target_id, ref variants)) = m.fusion {
        let fusion_idx = fusion_index as usize;
        if fusion_idx >= variants.len() {
            return Err("Invalid fusion type index".into());
        }

        let packed = &variants[fusion_idx];
        let mut merged_unit: NUnit = NUnit::unpack(packed)?;
        merged_unit = merged_unit.remap_ids(ctx).with_owner(pid);
        let merged_unit_id = merged_unit.id;
        merged_unit.clone().save(ctx)?;

        let mut team = m.team_load(ctx)?;

        // Delete source and target units
        if let Ok(source) = ctx.load::<NUnit>(source_id) {
            source.delete_recursive(ctx);
        }
        if let Ok(target) = ctx.load::<NUnit>(target_id) {
            target.delete_recursive(ctx);
        }

        // Replace target unit with merged unit in slots, remove source from slots
        let slots = team.slots_load(ctx)?;
        for slot in slots {
            if let Ok(slot_unit_id) = slot.unit_load_id(ctx) {
                let mut s = slot.clone();
                if slot_unit_id == target_id {
                    s.unit = Owned::new_id(merged_unit_id);
                } else if slot_unit_id == source_id {
                    s.unit = Owned::None;
                }
                s.clone().save(ctx)?;
            }
        }

        team.clone().save(ctx)?;
    } else {
        return Err("No fusion in progress".into());
    }

    m.fusion = None;
    m.clone().save(ctx)?;
    Ok(())
}

#[reducer]
fn match_cancel_fusion(ctx: &ReducerContext) -> Result<(), String> {
    let ctx = &mut ctx.as_context();
    let mut player = ctx.player()?;

    let mut m = player.active_match_load(ctx)?;
    m.fusion = None;
    m.clone().save(ctx)?;
    Ok(())
}

fn create_fused_unit(
    ctx: &mut ServerContext,
    pid: u64,
    source: &mut NUnit,
    target: &mut NUnit,
    fusion_type: i32,
) -> NodeResult<NUnit> {
    let source_name = source.unit_name.clone();
    let target_name = target.unit_name.clone();

    let source_state_stax = source.state_load(ctx)?.stax;
    let source_state_houses = source.house_ids(ctx);
    let source_reactions = source.behavior_load(ctx)?.reactions.clone();
    let source_desc = {
        let desc = source.description_load(ctx)?;
        desc.clone()
    };
    let source_rep = {
        let rep = source.representation_load(ctx)?;
        rep.clone()
    };
    let source_stats = {
        let stats = source.stats_load(ctx)?;
        stats.clone()
    };

    let target_state_stax = target.state_load(ctx)?.stax;
    let target_reactions = target.behavior_load(ctx)?.reactions.clone();
    let target_desc = {
        let desc = target.description_load(ctx)?;
        desc.clone()
    };
    let target_rep = {
        let rep = target.representation_load(ctx)?;
        rep.clone()
    };
    let target_stats = {
        let stats = target.stats_load(ctx)?;
        stats.clone()
    };

    let mut new_unit = NUnit::new(ctx.next_id(), pid, String::new());

    // Merge names
    let source_half = source_name.len() / 2;
    let target_half = target_name.len() / 2;
    new_unit.unit_name_set(format!(
        "{}{}",
        &source_name[..source_half],
        &target_name[target_name.len() - target_half..]
    ));

    // Merge descriptions
    let new_desc = NUnitDescription::new(
        ctx.next_id(),
        pid,
        format!("{} {}", source_desc.description, target_desc.description),
    );
    let desc_id = new_desc.id;
    new_desc.save(ctx)?;
    new_unit.description = Component::new_id(desc_id);

    // Merge behaviors based on fusion type
    let mut new_reactions = Vec::new();

    match fusion_type {
        0 => {
            // StickFront
            if let Some(source_reaction) = source_reactions.first() {
                let mut combined_actions = source_reaction.actions.clone();
                for reaction in &target_reactions {
                    combined_actions.extend(reaction.actions.clone());
                }
                new_reactions.push(Reaction {
                    trigger: source_reaction.trigger.clone(),
                    actions: combined_actions,
                });
            }
        }
        1 => {
            // StickBack
            if let Some(target_reaction) = target_reactions.first() {
                let mut combined_actions = target_reaction.actions.clone();
                for reaction in &source_reactions {
                    combined_actions.extend(reaction.actions.clone());
                }
                new_reactions.push(Reaction {
                    trigger: target_reaction.trigger.clone(),
                    actions: combined_actions,
                });
            }
        }
        2 => {
            // PushBack
            new_reactions.extend(source_reactions.clone());
            new_reactions.extend(target_reactions.clone());
        }
        _ => return Err("Invalid fusion type".into()),
    }

    let new_behavior = NUnitBehavior::new(ctx.next_id(), pid, new_reactions);
    let behavior_id = new_behavior.id;
    new_behavior.save(ctx)?;
    new_unit.behavior = Component::new_id(behavior_id);

    // Merge representations
    let mut merged_material = source_rep.material.clone();
    merged_material.0.extend(target_rep.material.0.clone());
    let new_rep = NUnitRepresentation::new(ctx.next_id(), pid, merged_material);
    let rep_id = new_rep.id;
    new_rep.save(ctx)?;
    new_unit.representation = Component::new_id(rep_id);

    // Merge stats
    let new_stats = NUnitStats::new(
        ctx.next_id(),
        pid,
        source_stats.pwr + target_stats.pwr,
        source_stats.hp + target_stats.hp,
    );
    let stats_id = new_stats.id;
    new_stats.save(ctx)?;
    new_unit.stats = Component::new_id(stats_id);

    for parent in source_state_houses {
        target.id.add_parent(ctx.rctx(), parent)?;
    }

    let new_state = NUnitState::new(ctx.next_id(), pid, source_state_stax + target_state_stax, 0);
    let state_id = new_state.id;
    new_state.save(ctx)?;
    new_unit.state = Component::new_id(state_id);

    let result_id = new_unit.id;
    new_unit.save(ctx)?;

    ctx.load::<NUnit>(result_id)
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
        vec![], // battle_history
        None,   // pending_battle
        None,   // fusion
    );
    let mut team = NTeam::new(ctx.next_id(), pid);
    for i in 0..gs.team_slots as i32 {
        let slot = NTeamSlot::new(ctx.next_id(), pid, i);
        team.slots_push(slot)?;
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
    // unlink_unit removed - no longer needed with new fusion system
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
