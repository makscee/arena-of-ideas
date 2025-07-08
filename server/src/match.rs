use std::i32;

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
        let price = ctx.global_settings().match_g.fusion_lvl_mul * fusion.lvl;
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
fn match_buy(ctx: &ReducerContext, i: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let g = m.g;
    if m.hand.len() >= 7 {
        return Err("Hand is full".into());
    }
    let offer = m
        .shop_offers
        .last_mut()
        .to_custom_e_s_fn(|| format!("No shop offers found"))?;
    let sc = offer.get_slot_mut(i)?;
    if sc.price > g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    let price = sc.price;
    let card_kind = sc.card_kind;
    let id = sc.node_id;
    m.hand.push((card_kind, id));
    m.g -= price;
    if let Some(limit) = &mut offer.buy_limit {
        *limit -= 1;
        if *limit == 0 {
            m.shop_offers.remove(m.shop_offers.len() - 1);
        }
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_play_house(ctx: &ReducerContext, i: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let i = i as usize;

    // First try to get from shop
    if let Some(offer) = m.shop_offers.last_mut() {
        if let Ok(shop_slot) = offer.get_slot_mut(i as u8) {
            if !shop_slot.sold && shop_slot.card_kind == CardKind::House {
                // Buy and play from shop
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

                let house =
                    id.to_node::<NHouse>(ctx)?
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
                return Ok(());
            }
        }
    }

    // Fallback to playing from hand
    let Some((card_kind, id)) = m.hand.get(i).copied() else {
        return Err(format!("Card {i} not found in hand or shop"));
    };
    m.hand.remove(i);
    if !matches!(card_kind, CardKind::House) {
        return Err(format!("Card {i} is not a house"));
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
fn match_play_unit(ctx: &ReducerContext, i: u8, slot: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let i = i as usize;

    // First try to get from shop
    let (card_kind, id) = if let Some(offer) = m.shop_offers.last_mut() {
        if let Ok(shop_slot) = offer.get_slot_mut(i as u8) {
            if !shop_slot.sold && shop_slot.card_kind == CardKind::Unit {
                // Buy from shop
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

                (CardKind::Unit, id)
            } else {
                // Fallback to playing from hand
                let Some((card_kind, id)) = m.hand.get(i).copied() else {
                    return Err(format!("Card {i} not found in hand or shop"));
                };
                m.hand.remove(i);
                (card_kind, id)
            }
        } else {
            // Fallback to playing from hand
            let Some((card_kind, id)) = m.hand.get(i).copied() else {
                return Err(format!("Card {i} not found in hand or shop"));
            };
            m.hand.remove(i);
            (card_kind, id)
        }
    } else {
        // Fallback to playing from hand
        let Some((card_kind, id)) = m.hand.get(i).copied() else {
            return Err(format!("Card {i} not found in hand or shop"));
        };
        m.hand.remove(i);
        (card_kind, id)
    };

    if !matches!(card_kind, CardKind::Unit) {
        return Err(format!("Card {i} is not a unit"));
    }
    let mut unit = NUnit::get(ctx, id)
        .to_custom_e_s_fn(|| format!("Failed to find Unit#{id}"))?
        .with_children(ctx)
        .with_components(ctx)
        .take();
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

    let slot = slot as i32;
    let fusion = m.get_slot_fusion(ctx, slot)?;
    let duplicate_unit = fusion.units_load(ctx)?.into_iter().find_map(|u| {
        if u.unit_name == unit.unit_name {
            Some(u)
        } else {
            None
        }
    });
    if let Some(mut unit) = duplicate_unit {
        let state = unit.state_load(ctx)?;
        state.xp += 1;
        while state.xp >= state.lvl {
            state.xp -= state.lvl;
            state.lvl += 1;
        }
        state.save(ctx);
    } else {
        let unit_id = unit.clone(ctx, pid, &mut default()).id;
        let mut unit = unit_id.to_node::<NUnit>(ctx)?;
        unit_id.add_parent(ctx, house_id)?;
        NUnitState::new(ctx, pid, 0, 1, 0)
            .id
            .add_child(ctx, unit_id)?;
        let fusion = m.get_slot_fusion(ctx, slot)?;
        fusion.units_add(ctx, unit_id)?;
        fusion.action_limit = fusion
            .action_limit
            .max(unit_tier as i32 * 2 + (fusion.lvl - 1) * 2);
        if fusion.units.ids.len() == 1 {
            let b = unit.description_load(ctx)?.behavior_load(ctx)?;
            fusion.behavior = b
                .reactions
                .iter()
                .enumerate()
                .map(|(t, r)| {
                    (
                        UnitTriggerRef {
                            unit: unit_id,
                            trigger: t as u8,
                        },
                        (0..r.actions.len() as u8)
                            .into_iter()
                            .map(|a| UnitActionRef {
                                unit: unit_id,
                                trigger: t as u8,
                                action: a,
                            })
                            .collect_vec(),
                    )
                })
                .collect();
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
    m.fill_shop_case(ctx)?;
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
    let fusion = NFusion::new(ctx, pid, default(), i32::MAX, 0, 0, 0, 1, 0, default());
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
    fusion.units.ids = unit_ids;

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
    let mut m = NMatch::new(
        ctx,
        pid,
        gs.match_g.initial,
        0,
        0,
        3,
        true,
        default(),
        default(),
    );
    m.id.add_child(ctx, player.id)?;
    let mut team = NTeam::new(ctx, pid);
    team.id.add_child(ctx, m.id)?;
    for i in 0..ctx.global_settings().team_slots as i32 {
        let fusion = NFusion::new(ctx, pid, default(), i, 0, 0, 0, 1, 0, default());
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
        let player_team_id = player_team.clone_ids_remap(ctx)?.id;
        player_team_id.add_parent(ctx, pool_id)?;
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
