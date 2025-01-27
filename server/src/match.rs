use std::i32;

use rand::seq::SliceRandom;

use super::*;

impl Match {
    fn register_update(c: &Context) -> Result<(), String> {
        let mut m: Match = NodeDomain::Match
            .tnode_filter_by_kind(c, NodeKind::Match)
            .into_iter()
            .next()
            .to_e_s("No matches found")?
            .to_node();
        m.last_update = Timestamp::now().into_micros_since_epoch();
        NodeDomain::Match.node_update(c, &m);
        Ok(())
    }
    fn get(c: &Context) -> Result<Match, String> {
        let id = NodeDomain::Match
            .tnode_filter_by_kind(c, NodeKind::Match)
            .get(0)
            .to_e_s("No matches found")?
            .id;
        let mut m = Match::from_table(c, NodeDomain::Match, id).to_e_s("Match not found")?;
        m.last_update = Timestamp::now().into_micros_since_epoch();
        Ok(m)
    }
    fn fill_case(&mut self, c: &Context) -> Result<(), String> {
        let price = c.global_settings().match_g.unit_buy;
        for slot in &mut self.shop_case {
            slot.sold = false;
            slot.price = price;
            slot.unit_id = NodeDomain::Alpha
                .tnode_filter_by_kind(c, NodeKind::Unit)
                .choose(&mut c.rc.rng())
                .to_e_s("No Alpha units found")?
                .id;
        }
        Ok(())
    }
    fn team(&self) -> Result<&Team, String> {
        self.team.as_ref().to_e_s("Team not set")
    }
    fn find_house<'a>(&'a self, name: &str) -> Option<&'a House> {
        self.team
            .as_ref()
            .unwrap()
            .houses
            .iter()
            .find(|h| h.name == name)
    }
    fn fill_gaps(c: &Context) {
        let mut units: Vec<Unit> = NodeDomain::Match.node_collect(c);
        units.sort_by_cached_key(|u| {
            NodeDomain::Match
                .tnode_find_by_key(c, &NodeKind::UnitSlot.key(u.id()))
                .map(|s| s.to_node::<UnitSlot>().slot)
                .unwrap_or(i32::MAX)
        });
        for (i, u) in units.into_iter().enumerate() {
            let node = UnitSlot {
                slot: i as i32,
                id: Some(u.id()),
            };
            NodeDomain::Match.node_insert_or_update(c, &node);
        }
    }
    fn save(&self, c: &Context) {
        NodeDomain::Match.node_update(c, self);
    }
}

impl House {
    fn find_ability<'a>(&'a self, name: &str) -> Option<&'a Ability> {
        self.abilities.iter().find(|a| a.name == name)
    }
}

#[reducer]
fn match_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let c = &ctx.wrap()?;
    let mut m = Match::get(c)?;
    let team_id = m.team()?.id();
    let occupied = NodeDomain::Match.node_collect::<UnitSlot>(c);
    if occupied.len() >= c.global_settings().team_slots as usize {
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
    NodeDomain::Match.node_update(c, sc);
    let mut unit =
        Unit::from_table(c, NodeDomain::Alpha, sc.unit_id).to_e_s("Failed to find Alpha unit")?;
    let mut ability: Ability = NodeDomain::Alpha.node_parent(c, sc.unit_id).unwrap();
    let mut house: House = NodeDomain::Alpha.node_parent(c, ability.id()).unwrap();

    unit.slot = Some(UnitSlot {
        slot: occupied.len() as i32,
        ..default()
    });
    unit.house_link.push(UnitHouseLink {
        name: house.name.clone(),
        ..default()
    });

    unit.clear_ids();
    ability.clear_ids();
    house.clear_ids();

    unit.to_table(c, NodeDomain::Match, team_id);
    if let Some(h) = m.find_house(&house.name) {
        if h.find_ability(&ability.name).is_none() {
            ability.to_table(c, NodeDomain::Match, h.id());
        }
    } else {
        house.abilities.push(ability);
        house.to_table(c, NodeDomain::Match, team_id);
    }
    m.save(c);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let c = &ctx.wrap()?;
    let slot = slot as i32;
    let slot = NodeDomain::Match
        .node_collect::<UnitSlot>(c)
        .into_iter()
        .find(|s| s.slot == slot)
        .to_e_s("Unit by slot not found")?;
    NodeDomain::Match.delete_by_id(c, slot.id());
    let mut m = Match::get(c)?;
    m.g += c.global_settings().match_g.unit_sell;
    Match::fill_gaps(c);
    m.save(c);
    Ok(())
}

#[reducer]
fn match_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let c = &ctx.wrap()?;
    let mut m = Match::get(c)?;
    let price = c.global_settings().match_g.reroll;
    if m.g < price {
        return Err("Not enough g".into());
    }
    m.g -= price;
    m.fill_case(c)?;
    m.save(c);
    for node in &m.shop_case {
        NodeDomain::Match.node_update(c, node);
    }
    Ok(())
}

#[reducer]
fn match_reorder(ctx: &ReducerContext, slot: u8, target: u8) -> Result<(), String> {
    let c = &ctx.wrap()?;
    let slot = slot as usize;
    let target = target as usize;
    let mut slots = NodeDomain::Match.node_collect::<UnitSlot>(c);
    if slot >= slots.len() {
        return Err("Slot outside of team length".into());
    }
    let target = target.min(slots.len() - 1);
    let unit = slots.remove(slot);
    slots.insert(target, unit);
    for (i, slot) in slots.iter_mut().enumerate() {
        slot.slot = i as i32;
        NodeDomain::Match.node_update(c, slot);
    }
    Match::register_update(c)
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let c = &ctx.wrap()?;
    let mut d = Match {
        g: 13,
        shop_case: (0..3).map(|_| default()).collect_vec(),
        team: Some(Team {
            name: "Test Team".into(),
            ..default()
        }),
        ..default()
    };
    d.fill_case(c)?;
    for d in NodeDomain::Match.tnode_collect_owner(c) {
        NodeDomain::Match.delete_by_id(c, d.id);
    }
    d.to_table(c, NodeDomain::Match, 0);
    Ok(())
}
