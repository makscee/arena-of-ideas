use rand::seq::SliceRandom;

use super::*;

impl Match {
    fn get(ctx: &ReducerContext) -> Result<Match, String> {
        let id = NodeDomain::Match
            .filter_by_kind(ctx, NodeKind::Match)
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
                .filter_by_kind(ctx, NodeKind::Unit)
                .choose(&mut ctx.rng())
                .to_e_s("No Alpha units found")?
                .id;
        }
        Ok(())
    }
}

#[reducer]
fn match_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    let slot = slot as usize;
    let sc = &mut m.shop_case[slot];
    let mut unit =
        Unit::from_table(ctx, NodeDomain::Alpha, sc.unit_id).to_e_s("Failed to find Alpha unit")?;
    unit.clear_ids();
    if m.team.as_ref().unwrap().units.len() >= GlobalSettings::get(ctx).team_slots as usize {
        return Err("Team already full".into());
    }
    if sc.sold {
        return Err("Unit already sold".into());
    }
    if sc.price > m.g {
        return Err("Not enough g".into());
    }
    sc.sold = true;
    m.g -= sc.price;
    unit.to_table(ctx, NodeDomain::Match, m.team.as_ref().unwrap().id.unwrap());
    NodeDomain::Match.update(ctx, sc);
    NodeDomain::Match.update(ctx, &m);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    let slot = slot as usize;
    let team = m.team.as_mut().to_e_s("Team not set")?;
    if slot >= team.units.len() {
        return Err("Slot index outside of team bounds".into());
    }
    let unit = team.units.remove(slot);
    NodeDomain::Match.delete(ctx, &unit);
    m.g += GlobalSettings::get(ctx).match_g.unit_sell;
    NodeDomain::Match.update(ctx, &m);
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
    NodeDomain::Match.update(ctx, &m);
    for node in &m.shop_case {
        NodeDomain::Match.update(ctx, node);
    }
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
    d.fill_case(ctx);
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
