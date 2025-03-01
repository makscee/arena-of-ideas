use std::{collections::HashMap, i32, ops::Deref};

use log::info;
use rand::seq::SliceRandom;

use super::*;

impl Match {
    fn register_update(ctx: &ReducerContext) -> Result<(), String> {
        // let mut m: Match = NodeDomain::Match
        //     .tnode_filter_by_kind(c, NodeKind::Match)
        //     .into_iter()
        //     .next()
        //     .to_e_s("No matches found")?
        //     .to_node();
        // m.last_update = Timestamp::now().into_micros_since_epoch();
        // NodeDomain::Match.node_update(c, &m);
        Ok(())
    }
    fn get(ctx: &ReducerContext) -> Result<Match, String> {
        // let id = NodeDomain::Match
        //     .tnode_filter_by_kind(c, NodeKind::Match)
        //     .get(0)
        //     .to_e_s("No matches found")?
        //     .id;
        // let mut m = Match::from_table(c, NodeDomain::Match, id).to_e_s("Match not found")?;
        // m.last_update = Timestamp::now().into_micros_since_epoch();
        Ok(default())
    }
    fn fill_case(&mut self, ctx: &ReducerContext) -> Result<(), String> {
        // let price = c.global_settings().match_g.unit_buy;
        // for slot in &mut self.shop_case {
        //     slot.sold = false;
        //     slot.price = price;
        //     slot.unit_id = NodeDomain::Core
        //         .tnode_filter_by_kind(c, NodeKind::Unit)
        //         .choose(&mut c.rc.rng())
        //         .to_e_s("No Core units found")?
        //         .id;
        // }
        Ok(())
    }
    fn find_house<'a>(&'a self, name: &str) -> Option<&'a House> {
        self.team().houses.iter().find(|h| h.name == name)
    }
    fn fill_gaps(ctx: &ReducerContext) {
        // let mut units: Vec<Unit> = NodeDomain::Match.node_collect(c);
        // units.sort_by_cached_key(|u| {
        //     NodeDomain::Match
        //         .tnode_find_by_key(c, &NodeKind::UnitSlot.key(u.id()))
        //         .map(|s| s.to_node::<UnitSlot>().slot)
        //         .unwrap_or(i32::MAX)
        // });
        // for (i, u) in units.into_iter().enumerate() {
        //     let node = UnitSlot {
        //         slot: i as i32,
        //         id: Some(u.id()),
        //     };
        //     NodeDomain::Match.node_insert_or_update(c, &node);
        // }
    }
    fn save(&self, ctx: &ReducerContext) {
        // NodeDomain::Match.node_update(c, self);
    }
}

impl House {
    fn find_action<'a>(&'a self, name: &str) -> Option<&'a ActionAbility> {
        self.action_abilities.iter().find(|a| a.name == name)
    }
    fn find_status<'a>(&'a self, name: &str) -> Option<&'a StatusAbility> {
        self.status_abilities.iter().find(|a| a.name == name)
    }
}

#[reducer]
fn match_buy(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?.iter_mut().next().unwrap();
    let g = m.g;
    let sc = m
        .shop_case_load(ctx)?
        .into_iter()
        .find(|s| s.id() == id)
        .to_e_s_fn(|| format!("Shop case slot not found for #{id}"))?;
    if sc.price > g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    let mut all = All::load(ctx);
    let unit = all
        .core_units(ctx)?
        .into_iter()
        .find(|u| u.name == sc.unit)
        .to_e_s_fn(|| format!("Failed to find unit {}", sc.unit))?;
    let mut house = unit.find_parent::<House>(ctx)?;
    let _ = m.team_load(ctx)?.houses_load(ctx);
    let houses = &mut m.team_mut().houses;
    if let Some(h) = houses.iter_mut().find(|h| h.name == house.name) {
        unit.clear_ids();
        h.units.push(unit.clone());
    } else {
        house.color_load(ctx)?;
        let _ = house.status_abilities_load(ctx);
        let _ = house.action_abilities_load(ctx);
        house.units.push(unit.clone());
        house.clear_ids();
        houses.push(house);
    }
    player.save(ctx);

    // let c = &ctx.wrap()?;
    // let mut m = Match::get(c)?;
    // let slot = slot as usize;
    // let sc = &mut m.shop_case[slot];
    // if sc.sold {
    //     return Err("Unit already sold".into());
    // }
    // if sc.price > m.g {
    //     return Err("Not enough g".into());
    // }
    // sc.sold = true;
    // m.g -= sc.price;
    // NodeDomain::Match.node_update(c, sc);
    // let mut unit =
    //     Unit::from_table(c, NodeDomain::Core, sc.unit_id).to_e_s("Failed to find Core unit")?;
    // let mut house: House = NodeDomain::Core.node_parent(c, unit.id()).unwrap();
    // unit.clear_ids();
    // if let Some(mut ability) = NodeDomain::Core.node_parent::<ActionAbility>(c, sc.unit_id) {
    //     if let Some(h) = m.find_house(&house.name) {
    //         if let Some(ability) = h.find_action(&ability.name) {
    //             unit.to_table(c, NodeDomain::Match, ability.id());
    //         } else {
    //             ability.clear_ids();
    //             ability.units.push(unit);
    //             ability.to_table(c, NodeDomain::Match, h.id());
    //         }
    //     } else {
    //         ability.clear_ids();
    //         house.clear_ids();
    //         ability.units.push(unit);
    //         house.action_abilities.push(ability);
    //         house.to_table(c, NodeDomain::Match, m.id());
    //     }
    // } else if let Some(mut ability) = NodeDomain::Core.node_parent::<StatusAbility>(c, sc.unit_id) {
    //     if let Some(h) = m.find_house(&house.name) {
    //         if let Some(ability) = h.find_status(&ability.name) {
    //             unit.to_table(c, NodeDomain::Match, ability.id());
    //         } else {
    //             ability.clear_ids();
    //             ability.units.push(unit);
    //             ability.to_table(c, NodeDomain::Match, h.id());
    //         }
    //     } else {
    //         ability.clear_ids();
    //         house.clear_ids();
    //         ability.units.push(unit);
    //         house.status_abilities.push(ability);
    //         house.to_table(c, NodeDomain::Match, m.id());
    //     }
    // } else {
    //     return Err("Ability not found".into());
    // }
    // m.save(c);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // if NodeDomain::Match
    //     .node_collect::<Fusion>(c)
    //     .into_iter()
    //     .any(|f| f.units.contains(&name))
    // {
    //     return Err("Can't sell fused unit".into());
    // }
    // if let Some(unit) = NodeDomain::Match
    //     .node_collect::<Unit>(c)
    //     .into_iter()
    //     .find(|u| u.name == name)
    // {
    //     NodeDomain::Match.delete_by_id_recursive(c, unit.id());
    // } else {
    //     return Err("Unit not found".into());
    // }
    // let mut m = Match::get(c)?;
    // m.g += c.global_settings().match_g.unit_sell;
    // Match::fill_gaps(c);
    // m.save(c);
    Ok(())
}

#[reducer]
fn match_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?.iter_mut().next().unwrap();
    let cost = ctx.global_settings().match_g.reroll;
    if m.g < cost {
        return Err("Not enough g".into());
    }
    m.g -= cost;
    let sc = m.shop_case_load(ctx)?;
    let mut all = All::load(ctx);
    let units = all.core_units(ctx)?;
    for c in sc {
        c.sold = false;
        c.unit = units.choose(&mut ctx.rng()).unwrap().name.clone();
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_reorder(ctx: &ReducerContext, slot: u8, target: u8) -> Result<(), String> {
    // let c = &ctx.wrap()?;
    // let slot = slot as usize;
    // let target = target as usize;
    // let mut slots = NodeDomain::Match.node_collect::<UnitSlot>(c);
    // if slot >= slots.len() {
    //     return Err("Slot outside of team length".into());
    // }
    // slots.sort_by_key(|s| s.slot);
    // let target = target.min(slots.len() - 1);
    // let unit = slots.remove(slot);
    // slots.insert(target, unit);
    // for (i, slot) in slots.iter_mut().enumerate() {
    //     slot.slot = i as i32;
    //     NodeDomain::Match.node_update(c, slot);
    // }
    // Match::register_update(c)
    Ok(())
}

#[reducer]
fn match_edit_fusions(ctx: &ReducerContext, fusions: Vec<Vec<String>>) -> Result<(), String> {
    // let c = &ctx.wrap()?;
    // let m = Match::get(c)?;
    // let fusions = fusions
    //     .into_iter()
    //     .map(|fusion| Fusion::from_strings(0, &fusion).unwrap())
    //     .collect_vec();
    // if fusions
    //     .iter()
    //     .any(|f| f.units.is_empty() || f.triggers.is_empty())
    // {
    //     return Err("Fusion can't be empty".into());
    // }
    // let roster_units: HashMap<String, Unit> = HashMap::from_iter(
    //     m.all_units()?
    //         .into_iter()
    //         .map(|u| (u.name.clone(), u.clone())),
    // );
    // info!("{fusions:?}");
    // for fusion in &m.team().fusions {
    //     NodeDomain::Match.delete_by_id_recursive(c, fusion.id());
    // }
    // for (i, mut fusion) in fusions.into_iter().enumerate() {
    //     fusion.slot = Some(UnitSlot::new(i as i32));
    //     fusion.to_table(c, NodeDomain::Match, m.team().id());
    // }
    // m.save(c);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    if let Ok(m) = player.active_match_load(ctx) {
        for m in m {
            m.delete_recursive(ctx)
        }
    }
    let mut all = All::load(ctx);
    let units = all.core_units(ctx)?;
    let gs = ctx.global_settings();
    let price = gs.match_g.unit_buy;
    let mut m = Match::new_full(
        gs.match_g.initial,
        Timestamp::now().into_micros_since_epoch(),
        Team::new("Test Team".into()),
    );
    m.shop_case = (0..3)
        .map(|_| {
            ShopCaseUnit::new(
                units.choose(&mut ctx.rng()).unwrap().name.clone(),
                price,
                false,
            )
        })
        .collect_vec();
    player.active_match = [m].into();
    player.save(ctx);
    Ok(())
}
