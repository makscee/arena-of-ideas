use rand::seq::SliceRandom;

use super::*;

#[reducer]
fn match_buy(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
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
    let unit = sc.unit;
    let price = sc.price;
    m.g -= price;
    let unit = Unit::get(ctx, unit).to_e_s_fn(|| format!("Failed to find Unit#{unit}"))?;
    let mut house = unit.find_parent::<House>(ctx)?;
    let houses = m.team_load(ctx)?.houses_load(ctx)?;
    if let Some(h) = houses.iter_mut().find(|h| h.name == house.name) {
        unit.clone(ctx, h.id);
    } else {
        house.color_load(ctx)?;
        let _ = house.status_abilities_load(ctx);
        let _ = house.action_abilities_load(ctx);
        let house = house.clone(ctx, m.team_load(ctx)?.id);
        unit.clone(ctx, house.id);
    }
    player.save(ctx);
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
        .find(|u| u.name == name)
        .to_e_s_fn(|| format!("Failed to find unit {name}"))?;
    unit.delete_recursive(ctx);
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
    let sc = m.shop_case_load(ctx)?;
    let mut all = All::load(ctx);
    let units = all.core_units(ctx)?;
    for c in sc {
        c.sold = false;
        c.unit = units.choose(&mut ctx.rng()).unwrap().id;
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_reorder(ctx: &ReducerContext, slot: u8, target: u8) -> Result<(), String> {
    // let mut player = ctx.player()?;
    // let m = player.active_match_load(ctx)?.iter_mut().next().unwrap();
    // let slot = slot as usize;
    // let target = target as usize;
    // let fusions = m.team_load(ctx)?.fusions_load(ctx)?;
    // fusions.sort_by_key(|f| f.slot);
    // if slot >= fusions.len() {
    //     return Err("Slot outside of team length".into());
    // }
    // let target = target.min(fusions.len() - 1);
    // let f = fusions.remove(slot);
    // fusions.insert(target, f);
    // for (i, slot) in fusions.iter_mut().enumerate() {
    //     slot.slot = i as i32;
    // }
    // debug!("{fusions:?}");
    // player.save(ctx);
    Ok(())
}

#[reducer]
fn match_edit_fusions(ctx: &ReducerContext, fusions: Vec<Vec<String>>) -> Result<(), String> {
    // let mut player = ctx.player()?;
    // let m = player.active_match_load(ctx)?.iter_mut().next().unwrap();
    // let fusions = fusions
    //     .into_iter()
    //     .map(|fusion| Fusion::from_strings(0, &fusion).unwrap())
    //     .collect_vec();
    // debug!("{fusions:?}");
    // if fusions
    //     .iter()
    //     .any(|f| f.units.is_empty() || f.triggers.is_empty())
    // {
    //     return Err("Fusion can't be empty".into());
    // }
    // let roster_units = m
    //     .roster_units_load(ctx)?
    //     .into_iter()
    //     .map(|u| &u.name)
    //     .collect_vec();
    // if let Some(unit) = fusions
    //     .iter()
    //     .find_map(|f| f.units.iter().find(|u| !roster_units.contains(u)))
    // {
    //     return Err(format!("Fusion unit {} not contained in roseter", unit));
    // }
    // let _ = m.team_load(ctx)?.fusions_load(ctx);
    // for f in &m.team().fusions {
    //     f.delete_recursive(ctx);
    // }
    // m.team_mut().fusions = fusions;
    // player.save(ctx);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    if let Ok(m) = player.active_match_load(ctx) {
        m.delete_recursive(ctx);
    }
    let mut all = All::load(ctx);
    let units = all.core_units(ctx)?;
    let gs = ctx.global_settings();
    let price = gs.match_g.unit_buy;
    let mut m = Match::new(ctx, player.id, gs.match_g.initial);
    let mut team = Team::new(ctx, m.id, "Test Team".into());
    team.fusions = (0..ctx.global_settings().team_slots)
        .map(|i| Fusion::new(ctx, team.id, i as i32, default(), default()))
        .collect();
    m.team = Some(team);
    m.shop_case = (0..3)
        .map(|_| {
            ShopCaseUnit::new(
                ctx,
                m.id,
                price,
                units.choose(&mut ctx.rng()).unwrap().id,
                false,
            )
        })
        .collect_vec();
    player.active_match = Some(m);
    player.save(ctx);
    Ok(())
}
