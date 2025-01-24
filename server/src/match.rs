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
        m.team_mut()?
            .units
            .sort_by_key(|u| u.slot.as_ref().unwrap().slot);
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
    fn team(&self) -> Result<&Team, String> {
        self.team.as_ref().to_e_s("Team not set")
    }
    fn team_mut(&mut self) -> Result<&mut Team, String> {
        self.team.as_mut().to_e_s("Team not set")
    }
    fn update_team_slots(&mut self, ctx: &ReducerContext) -> Result<(), String> {
        for (slot, unit) in self.team_mut()?.units.iter_mut().enumerate() {
            let node = unit.slot.as_mut().unwrap();
            node.slot = slot as i32;
            NodeDomain::Match.update(ctx, node);
        }
        Ok(())
    }
    fn reorder(&mut self, ctx: &ReducerContext, slot: usize, target: usize) -> Result<(), String> {
        let team = self.team_mut()?;
        if slot >= team.units.len() {
            return Err("Slot outside of team length".into());
        }
        let target = target.min(team.units.len() - 1);
        let unit = team.units.remove(slot);
        team.units.insert(target, unit);
        self.update_team_slots(ctx)
    }
    fn save(&self, ctx: &ReducerContext) {
        NodeDomain::Match.update(ctx, self);
    }
}

#[reducer]
fn match_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    let team = m.team()?;
    let unit_slot = team.units.len();
    if unit_slot >= GlobalSettings::get(ctx).team_slots as usize {
        return Err("Team already full".into());
    }
    let slot = slot as usize;
    let sc = &mut m.shop_case[slot];
    let mut unit =
        Unit::from_table(ctx, NodeDomain::Alpha, sc.unit_id).to_e_s("Failed to find Alpha unit")?;
    unit.slot = Some(UnitSlot {
        slot: unit_slot as i32,
        ..default()
    });
    unit.clear_ids();
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
    m.save(ctx);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    let slot = slot as usize;
    let team = m.team_mut()?;
    if slot >= team.units.len() {
        return Err("Slot index outside of team bounds".into());
    }
    let unit = team.units.remove(slot);
    NodeDomain::Match.delete(ctx, &unit);
    m.g += GlobalSettings::get(ctx).match_g.unit_sell;
    m.update_team_slots(ctx)?;
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
        NodeDomain::Match.update(ctx, node);
    }
    Ok(())
}

#[reducer]
fn match_reorder(ctx: &ReducerContext, slot: u8, target: u8) -> Result<(), String> {
    let mut m = Match::get(ctx)?;
    m.reorder(ctx, slot as usize, target as usize)?;
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
