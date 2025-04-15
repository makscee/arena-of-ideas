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
    let unit = Unit::get(ctx, unit)
        .to_e_s_fn(|| format!("Failed to find Unit#{unit}"))?
        .with_children(ctx)
        .with_components(ctx)
        .take();
    let mut house = unit.find_parent::<House>(ctx)?;
    let team = m.team_load(ctx)?;
    let _ = team.houses_load(ctx);
    let houses = &mut team.houses;
    if let Some(h) = houses.iter_mut().find(|h| h.house_name == house.house_name) {
        unit.clone(ctx, h.id);
    } else {
        house.color_load(ctx)?;
        let _ = house.status_ability_load(ctx);
        let _ = house.action_ability_load(ctx);
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
        .find(|u| u.unit_name == name)
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
    let mut core = Core::load(ctx);
    let units = core.all_units(ctx)?;
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
fn match_edit_fusions(ctx: &ReducerContext, fusions: Vec<String>) -> Result<(), String> {
    let mut player = ctx.player()?;
    let m = player.active_match_load(ctx)?;
    let team = m.team_load(ctx)?;
    let _ = team.fusions_load(ctx);
    for fusion in std::mem::take(&mut team.fusions) {
        fusion.delete_recursive(ctx);
    }
    for s in fusions {
        let fusion: Fusion = ron::from_str(&s).map_err(|e| e.to_string())?;
        fusion.clone(ctx, team.id());
    }
    player.save(ctx);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    if let Ok(m) = player.active_match_load(ctx) {
        m.delete_recursive(ctx);
    }
    let mut core = Core::load(ctx);
    let units = core.all_units(ctx)?;
    let gs = ctx.global_settings();
    let price = gs.match_g.unit_buy;
    let mut m = Match::new(ctx, player.id, gs.match_g.initial);
    let team = Team::new(ctx, m.id, "New Team".into());
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
