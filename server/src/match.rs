use std::i32;

use rand::seq::SliceRandom;

use super::*;

impl Match {
    fn get(ctx: &ReducerContext) -> Result<Match, String> {
        let id = NodeDomain::Match
            .tnode_filter_by_kind(ctx, NodeKind::Match)
            .get(0)
            .to_e_s("No matches found")?
            .id;
        let mut m = Match::from_table(ctx, NodeDomain::Match, id).to_e_s("Match not found")?;
        m.last_update = Timestamp::now().into_micros_since_epoch();
        Ok(m)
    }
    fn fill_case(&mut self, ctx: &ReducerContext) -> Result<(), String> {
        let price = GlobalSettings::get(ctx).match_g.unit_buy;
        for slot in &mut self.shop_case {
            slot.sold = false;
            slot.price = price;
            slot.unit_id = NodeDomain::Alpha
                .tnode_filter_by_kind(ctx, NodeKind::Unit)
                .choose(&mut ctx.rng())
                .to_e_s("No Alpha units found")?
                .id;
        }
        Ok(())
    }
    fn team(&self) -> Result<&Team, String> {
        self.team.as_ref().to_e_s("Team not set")
    }
    fn team_mut(&mut self) -> Result<&mut Team, String> {
        self.team.as_mut().to_e_s("Team not set")
    }
    fn find_house<'a>(&'a self, name: &str) -> Option<&'a House> {
        self.team
            .as_ref()
            .unwrap()
            .houses
            .iter()
            .find(|h| h.name == name)
    }
    fn fill_gaps(ctx: &ReducerContext) {
        let mut units: Vec<Unit> = NodeDomain::Match.node_collect(ctx);
        units.sort_by_cached_key(|u| {
            NodeDomain::Match
                .tnode_find_by_key(ctx, &NodeKind::UnitSlot.key(u.id()))
                .map(|s| s.to_node::<UnitSlot>().slot)
                .unwrap_or(i32::MAX)
        });
        for (i, u) in units.into_iter().enumerate() {
            let node = UnitSlot {
                slot: i as i32,
                id: Some(u.id()),
            };
            NodeDomain::Match.node_insert_or_update(ctx, &node);
        }
    }
    // fn update_team_slots(&mut self, ctx: &ReducerContext) -> Result<(), String> {
    //     for (slot, unit) in self.team_mut()?.units.iter_mut().enumerate() {
    //         let node = unit.slot.as_mut().unwrap();
    //         node.slot = slot as i32;
    //         NodeDomain::Match.update(ctx, node);
    //     }
    //     Ok(())
    // }
    // fn reorder(&mut self, ctx: &ReducerContext, slot: usize, target: usize) -> Result<(), String> {
    //     let team = self.team_mut()?;
    //     if slot >= team.units.len() {
    //         return Err("Slot outside of team length".into());
    //     }
    //     let target = target.min(team.units.len() - 1);
    //     let unit = team.units.remove(slot);
    //     team.units.insert(target, unit);
    //     self.update_team_slots(ctx)
    // }
    fn save(&self, ctx: &ReducerContext) {
        NodeDomain::Match.node_update(ctx, self);
    }
}

impl House {
    fn find_ability<'a>(&'a self, name: &str) -> Option<&'a Ability> {
        self.abilities.iter().find(|a| a.name == name)
    }
}

#[reducer]
fn match_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    let team_id = m.team()?.id();
    let occupied = NodeDomain::Match.node_collect::<UnitSlot>(ctx);
    if occupied.len() >= GlobalSettings::get(ctx).team_slots as usize {
        return Err("Team already full".into());
    }
    let slot = slot as usize;
    let sc = &mut m.shop_case[slot];
    if sc.sold {
        return Err("Unit already sold".into());
    }
    if sc.price > m.g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    m.g -= sc.price;
    NodeDomain::Match.node_update(ctx, sc);
    let mut unit =
        Unit::from_table(ctx, NodeDomain::Alpha, sc.unit_id).to_e_s("Failed to find Alpha unit")?;
    unit.slot = Some(UnitSlot {
        slot: occupied.len() as i32,
        ..default()
    });
    let mut ability: Ability = NodeDomain::Alpha.node_parent(ctx, sc.unit_id).unwrap();
    let mut house: House = NodeDomain::Alpha.node_parent(ctx, ability.id()).unwrap();
    unit.clear_ids();
    ability.clear_ids();
    house.clear_ids();
    if let Some(h) = m.find_house(&house.name) {
        if let Some(a) = h.find_ability(&ability.name) {
            unit.to_table(ctx, NodeDomain::Match, a.id());
        } else {
            ability.units.push(unit);
            ability.to_table(ctx, NodeDomain::Match, h.id());
        }
    } else {
        ability.units.push(unit);
        house.abilities.push(ability);
        house.to_table(ctx, NodeDomain::Match, team_id);
    }
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let slot = slot as i32;
    let slot = NodeDomain::Match
        .node_collect::<UnitSlot>(ctx)
        .into_iter()
        .find(|s| s.slot == slot)
        .to_e_s("Unit by slot not found")?;
    NodeDomain::Match.delete_by_id(ctx, slot.id());
    let mut m = Match::get(ctx)?;
    m.g += GlobalSettings::get(ctx).match_g.unit_sell;
    Match::fill_gaps(ctx);
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    let price = GlobalSettings::get(ctx).match_g.reroll;
    if m.g < price {
        return Err("Not enough g".into());
    }
    m.g -= price;
    m.fill_case(ctx)?;
    m.save(ctx);
    for node in &m.shop_case {
        NodeDomain::Match.node_update(ctx, node);
    }
    Ok(())
}

#[reducer]
fn match_reorder(ctx: &ReducerContext, slot: u8, target: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    // m.reorder(ctx, slot as usize, target as usize)?;
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut d = Match {
        g: 13,
        shop_case: (0..3).map(|_| default()).collect_vec(),
        team: Some(Team {
            name: "Test Team".into(),
            ..default()
        }),
        ..default()
    };
    d.fill_case(ctx)?;
    for d in ctx.db.nodes_match().iter() {
        ctx.db.nodes_match().key().delete(d.key);
    }
    d.to_table(ctx, NodeDomain::Match, 0);
    Ok(())
}

#[reducer]
fn match_get(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let d = Match::from_table(ctx, NodeDomain::Match, id);
    log::info!("{d:?}");
    Ok(())
}
