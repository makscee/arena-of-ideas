use std::i32;

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
        let house = house.with_components(ctx).clone(ctx, team.id);
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
fn match_buy_fusion(ctx: &ReducerContext) -> Result<(), String> {
    let mut player = ctx.player()?;
    let team = player.active_match_load(ctx)?.team_load(ctx)?;
    let _ = team.fusions_load(ctx);
    if team.fusions.len() >= ctx.global_settings().team_slots as usize {
        return Err("Team size limit reached".into());
    }
    let fusion = Fusion::new(ctx, team.id(), i32::MAX, default(), default());
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
    let fusion: Fusion = fusion.to_node()?;
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
