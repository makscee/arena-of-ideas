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
    let unit = sc.unit.clone();
    let price = sc.price;
    m.g -= price;
    let mut all = All::load(ctx);
    let unit = all
        .core_units(ctx)?
        .into_iter()
        .find(|u| u.name == unit)
        .to_e_s_fn(|| format!("Failed to find unit {}", unit))?;
    let mut house = unit.find_parent::<House>(ctx)?;
    let _ = m.team_load(ctx)?.houses_load(ctx);
    let houses = &mut m.team_mut().houses;
    let mut unit = unit.clone().with_children(ctx).with_components(ctx);
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
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?.iter_mut().next().unwrap();
    m.g += ctx.global_settings().match_g.unit_sell;
    m.update_self(ctx);
    let unit = m
        .roster_units_load(ctx)?
        .into_iter()
        .find(|u| u.name == name)
        .to_e_s_fn(|| format!("Failed to find unit {name}"))?;
    unit.delete_recursive(ctx);
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
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?.iter_mut().next().unwrap();
    let fusions = fusions
        .into_iter()
        .map(|fusion| Fusion::from_strings(0, &fusion).unwrap())
        .collect_vec();
    debug!("{fusions:?}");
    if fusions
        .iter()
        .any(|f| f.units.is_empty() || f.triggers.is_empty())
    {
        return Err("Fusion can't be empty".into());
    }
    let roster_units = m
        .roster_units_load(ctx)?
        .into_iter()
        .map(|u| &u.name)
        .collect_vec();
    if let Some(unit) = fusions
        .iter()
        .find_map(|f| f.units.iter().find(|u| !roster_units.contains(u)))
    {
        return Err(format!("Fusion unit {} not contained in roseter", unit));
    }
    let _ = m.team_load(ctx)?.fusions_load(ctx);
    for f in &m.team().fusions {
        f.delete_recursive(ctx);
    }
    m.team_mut().fusions = fusions;
    player.save(ctx);
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
