use std::{collections::HashMap, i32};

use rand::{Rng, seq::SliceRandom};

use super::*;

use raw_nodes::NodeKind;

impl NMatch {
    fn fill_shop_case(&mut self, ctx: &ReducerContext) -> Result<(), String> {
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
                let unit = ctx.rng().gen_bool(0.5);
                let n =
                    if unit && !units_from_owned_houses.is_empty() || not_owned_houses.is_empty() {
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
    fn get_slot_fusion(&mut self, ctx: &ReducerContext, slot: i32) -> Result<&mut NFusion, String> {
        self.team_load(ctx)?
            .fusions_load(ctx)?
            .into_iter()
            .find(|f| f.slot == slot)
            .to_custom_e_s_fn(|| format!("Failed to find Fusion in slot {slot}"))
    }
    fn buy_fusion_lvl(&mut self, ctx: &ReducerContext, slot: usize) -> Result<(), String> {
        let fusion = self.get_slot_fusion(ctx, slot as i32)?;
        fusion.lvl += 1;
        let price = ctx.global_settings().match_g.fusion_slot_mul * fusion.lvl;
        if self.g < price {
            return Err("Not enough g".into());
        }
        self.g -= price;
        Ok(())
    }
}

impl NFusion {
    fn units_load(&self, ctx: &ReducerContext) -> Result<Vec<NUnit>, String> {
        let mut result: Vec<NUnit> = default();
        for id in &self.units.ids {
            result.push(id.to_node(ctx)?);
        }
        Ok(result)
    }
}

#[reducer]
fn match_play_house(ctx: &ReducerContext, i: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;

    let offer = m
        .shop_offers
        .last_mut()
        .to_custom_e_s_fn(|| format!("No shop offers found"))?;
    let shop_slot = offer.get_slot_mut(i)?;

    if shop_slot.sold {
        return Err("Item already sold".into());
    }

    if shop_slot.card_kind != CardKind::House {
        return Err("Item is not a house".into());
    }

    let g = m.g;
    if shop_slot.price > g {
        return Err("Not enough g".into());
    }

    shop_slot.sold = true;
    let price = shop_slot.price;
    let id = shop_slot.node_id;
    m.g -= price;

    if let Some(limit) = &mut offer.buy_limit {
        *limit -= 1;
        if *limit == 0 {
            m.shop_offers.remove(m.shop_offers.len() - 1);
        }
    }

    let house = id
        .to_node::<NHouse>(ctx)?
        .with_components(ctx)
        .clone(ctx, pid, &mut default());
    house.id.add_parent(ctx, m.team_load(ctx)?.id)?;
    let house_units = id
        .collect_kind_children(ctx, NodeKind::NUnit)
        .choose_multiple(&mut ctx.rng(), 3)
        .copied()
        .collect_vec();
    m.shop_offers.push(ShopOffer {
        buy_limit: Some(1),
        case: ShopSlot::units_from_ids(house_units, 0),
    });
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_buy_fusion_lvl(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    m.buy_fusion_lvl(ctx, slot as usize)?;
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_buy_unit(ctx: &ReducerContext, shop_idx: u8, slot: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let offer = m
        .shop_offers
        .last_mut()
        .to_custom_e_s_fn(|| format!("No shop offers found"))?;
    let shop_slot = offer.get_slot_mut(shop_idx)?;

    if shop_slot.sold {
        return Err("Item already sold".into());
    }

    if shop_slot.card_kind != CardKind::Unit {
        return Err("Item is not a unit".into());
    }

    let g = m.g;
    if shop_slot.price > g {
        return Err("Not enough g".into());
    }

    shop_slot.sold = true;
    let price = shop_slot.price;
    let unit_id = shop_slot.node_id;
    m.g -= price;

    if let Some(limit) = &mut offer.buy_limit {
        *limit -= 1;
        if *limit == 0 {
            m.shop_offers.remove(m.shop_offers.len() - 1);
        }
    }

    let unit = NUnit::get(ctx, unit_id)
        .to_custom_e_s_fn(|| format!("Failed to find Unit#{unit_id}"))?
        .with_children(ctx)
        .with_components(ctx)
        .take();

    let mut player = ctx.player()?;
    let mut m = player.active_match_load(ctx)?;
    match_move_unit_to_slot(ctx, &mut m, None, unit, slot as i32, -1, true)
}

#[reducer]
fn match_buy_unit_allow_stack(ctx: &ReducerContext, shop_idx: u8, slot: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let offer = m
        .shop_offers
        .last_mut()
        .to_custom_e_s_fn(|| format!("No shop offers found"))?;
    let shop_slot = offer.get_slot_mut(shop_idx)?;

    if shop_slot.sold {
        return Err("Item already sold".into());
    }

    if shop_slot.card_kind != CardKind::Unit {
        return Err("Item is not a unit".into());
    }

    let g = m.g;
    if shop_slot.price > g {
        return Err("Not enough g".into());
    }

    shop_slot.sold = true;
    let price = shop_slot.price;
    let unit_id = shop_slot.node_id;
    m.g -= price;

    if let Some(limit) = &mut offer.buy_limit {
        *limit -= 1;
        if *limit == 0 {
            m.shop_offers.remove(m.shop_offers.len() - 1);
        }
    }

    let unit = NUnit::get(ctx, unit_id)
        .to_custom_e_s_fn(|| format!("Failed to find Unit#{unit_id}"))?
        .with_children(ctx)
        .with_components(ctx)
        .take();

    let mut player = ctx.player()?;
    let mut m = player.active_match_load(ctx)?;
    match_move_unit_to_slot(ctx, &mut m, None, unit, slot as i32, -1, false)
}

#[reducer]
fn match_move_owned_unit(
    ctx: &ReducerContext,
    source_unit_id: u64,
    target_fusion_slot: i32,
    target_position: i32,
) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let team = m.team_load(ctx)?;
    let all_fusions = team.fusions_load(ctx)?;

    let mut source_fusion_id = None;
    let mut source_position = None;

    for fusion in all_fusions.iter() {
        let units = fusion.units_load(ctx)?;
        for (pos, unit) in units.iter().enumerate() {
            if unit.id == source_unit_id {
                source_fusion_id = Some(fusion.id);
                source_position = Some(pos);
                break;
            }
        }
        if source_fusion_id.is_some() {
            break;
        }
    }

    let source_fusion_id = source_fusion_id.to_custom_e_s("Source fusion not found")?;
    let source_position = source_position.to_custom_e_s("Source position not found")?;

    // Get the source unit before removing it
    let source_unit = NUnit::get(ctx, source_unit_id)
        .to_custom_e_s("Source unit not found")?
        .with_children(ctx)
        .with_components(ctx)
        .take();

    // Remove unit from source fusion
    for fusion in all_fusions.iter_mut() {
        if fusion.id == source_fusion_id {
            fusion.units_remove(ctx, source_unit_id)?;
            // Also remove behavior at this position
            if source_position < fusion.behavior.len() {
                fusion.behavior.remove(source_position);
            }
            break;
        }
    }

    let mut player = ctx.player()?;
    let mut m = player.active_match_load(ctx)?;
    match_move_unit_to_slot(
        ctx,
        &mut m,
        Some(source_unit_id),
        source_unit,
        target_fusion_slot,
        target_position,
        false,
    )
}

fn match_move_unit_to_slot(
    ctx: &ReducerContext,
    m: &mut NMatch,
    source_unit_id: Option<u64>,
    mut unit: NUnit,
    target_fusion_slot: i32,
    target_position: i32,
    from_shop_only_empty: bool,
) -> Result<(), String> {
    let player = ctx.player()?;
    let pid = player.id;

    let unit_tier = unit
        .description_load(ctx)?
        .behavior_load(ctx)?
        .reactions
        .tier();

    let team = m.team_load(ctx)?;
    let _ = team.houses_load(ctx);
    let house = unit
        .find_parent::<NHouse>(ctx)
        .to_custom_e_s("Failed to find House parent of Unit")?
        .house_name;
    let house_id = team
        .houses
        .iter()
        .find(|h| h.house_name == house)
        .map(|h| h.id)
        .to_custom_e_s_fn(|| format!("Team house {house} not found"))?;

    let target_fusion = m.get_slot_fusion(ctx, target_fusion_slot)?;
    let mut target_units = target_fusion.units_load(ctx)?;

    // Check if target position is occupied
    let target_slot_idx = if target_position < 0 {
        target_units.len()
    } else {
        target_position as usize
    };

    if target_slot_idx < target_units.len() {
        let target_unit = &target_units[target_slot_idx];

        // Check if we can stack (same unit name)
        if target_unit.unit_name == unit.unit_name {
            // Stack the units
            let target_unit = &mut target_units[target_slot_idx];
            let target_state = target_unit.state_load(ctx)?;
            let xp = unit
                .state_load(ctx)
                .map(|s| s.xp + ((1.0 + (s.lvl - 1) as f32) / 2.0 * (s.lvl - 1) as f32) as i32 + 1)
                .unwrap_or(1);
            target_state.xp += xp;
            while target_state.xp >= target_state.lvl {
                target_state.xp -= target_state.lvl;
                target_state.lvl += 1;
            }
            target_state.update_self(ctx);
            unit.delete_with_components(ctx);
            m.save(ctx);
            return Ok(());
        } else {
            // Different units - check if we can swap
            if from_shop_only_empty {
                return Err("Can only buy units into empty slots or stack with same unit".into());
            }

            // Swap positions if moving owned unit
            if let Some(source_id) = source_unit_id {
                // This is a swap - the source unit was already removed from its fusion
                // We need to move the target unit to where the source came from
                let unit_id = if source_id != unit.id {
                    unit.clone(ctx, pid, &mut HashMap::default()).id
                } else {
                    source_id
                };

                // Insert the unit at target position
                let _unit_clone = unit_id.to_node::<NUnit>(ctx)?;
                if source_id != unit.id {
                    unit_id.add_parent(ctx, house_id)?;
                    let mut unit_state = NUnitState::new(pid, 0, 1, 0);
                    unit_state.insert_self(ctx);
                    unit_state.id.add_child(ctx, unit_id)?;
                }

                // Remove target unit from current position and add source unit
                let target_unit_id = target_unit.id;
                target_fusion.units_remove(ctx, target_unit_id)?;
                target_fusion.units.ids.insert(target_slot_idx, unit_id);
                unit_id.add_parent(ctx, target_fusion.id)?;

                // Update behavior
                if target_slot_idx < target_fusion.behavior.len() {
                    target_fusion.behavior.remove(target_slot_idx);
                }
                let b = unit.description_load(ctx)?.behavior_load(ctx)?;
                target_fusion.behavior.insert(
                    target_slot_idx,
                    UnitActionRange {
                        trigger: 0,
                        start: 0,
                        length: b.reactions.get(0).map(|r| r.actions.len()).unwrap_or(0) as u8,
                    },
                );

                target_fusion.action_limit = target_fusion
                    .action_limit
                    .max(unit_tier as i32 * 2 + (target_fusion.lvl - 1) * 2);

                m.save(ctx);
                return Ok(());
            } else {
                return Err("Cannot replace existing unit when buying from shop".into());
            }
        }
    } else {
        // Empty slot - place unit here
        if target_fusion.units.ids.len() >= target_fusion.lvl as usize {
            return Err("Target fusion is full".into());
        }

        let unit_id = if let Some(source_id) = source_unit_id {
            source_id
        } else {
            let new_id = unit.clone(ctx, pid, &mut HashMap::default()).id;
            new_id.add_parent(ctx, house_id)?;
            let mut unit_state = NUnitState::new(pid, 0, 1, 0);
            unit_state.insert_self(ctx);
            unit_state.id.add_child(ctx, new_id)?;
            new_id
        };

        target_fusion.units_add(ctx, unit_id)?;
        target_fusion.action_limit = target_fusion
            .action_limit
            .max(unit_tier as i32 * 2 + (target_fusion.lvl - 1) * 2);

        if target_fusion.units.ids.len() == 1 {
            target_fusion.trigger.unit = unit.id;
        }

        let b = unit.description_load(ctx)?.behavior_load(ctx)?;
        let units = target_fusion.units_load(ctx)?;
        if let Some(unit_index) = units.iter().position(|u| u.id == unit_id) {
            target_fusion.behavior.resize(
                units.len(),
                UnitActionRange {
                    trigger: 0,
                    start: 0,
                    length: 0,
                },
            );

            target_fusion.behavior[unit_index] = UnitActionRange {
                trigger: 0,
                start: 0,
                length: b.reactions.get(0).map(|r| r.actions.len()).unwrap_or(0) as u8,
            };
        }
    }

    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_sell_fusion_unit(
    ctx: &ReducerContext,
    fusion_id: u64,
    unit_id: u64,
) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;

    // Give the player sell gold
    m.g += ctx.global_settings().match_g.unit_sell;

    // Find the fusion and remove the unit
    let fusion = m
        .team_load(ctx)?
        .fusions_load(ctx)?
        .into_iter()
        .find(|f| f.id == fusion_id)
        .to_custom_e_s_fn(|| format!("Failed to find Fusion#{fusion_id}"))?;

    let units = fusion.units_load(ctx)?;
    if let Some(unit_index) = units.iter().position(|u| u.id == unit_id) {
        if unit_index < fusion.behavior.len() {
            fusion.behavior.remove(unit_index);
        }
    }

    fusion.units_remove(ctx, unit_id)?;

    // Delete the unit completely
    let unit = units
        .into_iter()
        .find(|u| u.id == unit_id)
        .to_custom_e_s_fn(|| format!("Failed to find Unit#{unit_id}"))?;
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
    m.fill_shop_case(ctx)?;
    player.save(ctx);
    Ok(())
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
fn match_reorder_fusion_units(
    ctx: &ReducerContext,
    fusion_id: u64,
    unit_ids: Vec<u64>,
) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let fusion = m
        .team_load(ctx)?
        .fusions_load(ctx)?
        .into_iter()
        .find(|f| f.id == fusion_id)
        .to_custom_e_s_fn(|| format!("Failed to find Fusion#{fusion_id}"))?;

    let current_units = fusion.units.ids.clone();

    // Validate that the provided unit_ids match the current units
    if unit_ids.len() != current_units.len() {
        return Err("Wrong number of units provided".into());
    }

    // Check for duplicates
    if let Some(id) = unit_ids.iter().duplicates().next() {
        return Err(format!("Duplicate unit id#{id}"));
    }

    // Check that all provided units are currently in the fusion
    for unit_id in &unit_ids {
        if !current_units.contains(unit_id) {
            return Err(format!("Unit#{unit_id} is not in Fusion#{fusion_id}"));
        }
    }

    // Update the fusion with the new unit order
    fusion.units.ids = unit_ids.clone();

    let old_behavior = fusion.behavior.clone();
    fusion.behavior.clear();

    for unit_id in &unit_ids {
        if let Some(old_index) = current_units.iter().position(|id| id == unit_id) {
            let behavior = old_behavior
                .get(old_index)
                .cloned()
                .unwrap_or(UnitActionRange {
                    trigger: 0,
                    start: 0,
                    length: 0,
                });
            fusion.behavior.push(behavior);
        }
    }

    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_move_unit_between_fusions(
    ctx: &ReducerContext,
    source_fusion_id: u64,
    target_fusion_id: u64,
    unit_id: u64,
    target_slot_idx: u32,
) -> Result<(), String> {
    if source_fusion_id == target_fusion_id {
        return Err("Cannot move unit within same fusion using this function".into());
    }

    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let team = m.team_load(ctx)?;
    let fusions = team.fusions_load(ctx)?;

    let target_fusion = fusions
        .iter()
        .find(|f| f.id == target_fusion_id)
        .ok_or_else(|| format!("Failed to find target Fusion#{}", target_fusion_id))?;

    let target_position = if target_slot_idx as usize >= target_fusion.units.ids.len() {
        -1 // Append to end
    } else {
        target_slot_idx as i32
    };

    match_move_owned_unit(ctx, unit_id, target_fusion.slot, target_position)
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    for m in NMatch::collect_owner(ctx, player.id) {
        m.delete_with_components(ctx);
    }
    let gs = ctx.global_settings();
    let mut m = NMatch::new(pid, gs.match_g.initial, 0, 0, 3, true, default());
    m.insert_self(ctx);
    m.id.add_child(ctx, player.id)?;
    let mut team = NTeam::new(pid);
    team.insert_self(ctx);
    team.id.add_child(ctx, m.id)?;
    for i in 0..ctx.global_settings().team_slots as i32 {
        let mut fusion = NFusion::new(pid, default(), i, 0, 0, 0, 1, 0, default(), default());
        fusion.insert_self(ctx);
        fusion.id.add_parent(ctx, team.id())?;
        team.fusions.push(fusion);
    }
    m.team = Some(team);
    m.fill_shop_case(ctx)?;
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
        let mut new_pool = NFloorPool::new(ID_ARENA, floor);
        new_pool.insert_self(ctx);
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
        let player_team_id = player_team.clone_ids_remap(ctx)?.id;
        player_team_id.add_parent(ctx, pool_id)?;
        let mut battle = NBattle::new(
            pid,
            player_team_id,
            team.id,
            ctx.timestamp.to_micros_since_unix_epoch() as u64,
            default(),
            None,
        );
        battle.insert_self(ctx);
        battle.id.add_parent(ctx, m_id)?;
    } else {
        let _ = arena.floor_bosses_load(ctx);
        let mut floor_boss = NFloorBoss::new(ID_ARENA, floor);
        floor_boss.insert_self(ctx);
        floor_boss.id.add_parent(ctx, arena.id)?;
        player_team
            .clone_ids_remap(ctx)?
            .id
            .add_child(ctx, floor_boss.id)?;
        player_team
            .clone_ids_remap(ctx)?
            .id
            .add_parent(ctx, pool_id)?;
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
fn match_set_fusion_unit_action_range(
    ctx: &ReducerContext,
    unit_id: u64,
    actions_start: u8,
    actions_len: u8,
) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let team = m.team_load(ctx)?;

    // Find the fusion containing this unit
    let mut fusion_found = false;
    for fusion in team.fusions_load(ctx)? {
        let mut units = fusion.units_load(ctx)?;

        // Find the index of the unit in this fusion
        if let Some(unit_index) = units.iter().position(|u| u.id == unit_id) {
            let unit = &mut units[unit_index];
            let description = unit.description_load(ctx)?;
            let unit_behavior = description.behavior_load(ctx)?;

            // Find the maximum number of actions available for any trigger
            let max_actions = unit_behavior
                .reactions
                .iter()
                .map(|r| r.actions.len() as u8)
                .max()
                .unwrap_or(0);

            // Validate range bounds
            if actions_start >= max_actions {
                return Err(format!(
                    "Start index {} exceeds available actions {}",
                    actions_start, max_actions
                ));
            }

            let adjusted_len = if actions_start + actions_len > max_actions {
                max_actions.saturating_sub(actions_start)
            } else {
                actions_len
            };

            // Ensure behavior vector has the same length as units
            fusion.behavior.resize(
                units.len(),
                UnitActionRange {
                    trigger: 0,
                    start: 0,
                    length: 0,
                },
            );

            // Update the action range for this unit at the corresponding index
            fusion.behavior[unit_index].start = actions_start;
            fusion.behavior[unit_index].length = adjusted_len;
            fusion_found = true;
            break;
        }
    }

    if !fusion_found {
        return Err("Unit not found in any fusion".into());
    }

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
