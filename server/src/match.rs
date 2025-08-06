use super::*;

#[reducer]
fn match_shop_buy(ctx: &ReducerContext, shop_idx: u8) -> Result<(), String> {
    let mut player = ctx.player()?;
    let pid = player.id;
    let m = player.active_match_load(ctx)?;
    let offer = m
        .shop_offers
        .last_mut()
        .to_custom_e_s("No active shop offers")?;
    let slot = offer
        .case
        .get_mut(shop_idx as usize)
        .to_custom_e_s_fn(|| format!("Shop slot {shop_idx} not found"))?;
    if slot.sold {
        return Err("Shop slot already sold".to_string());
    }
    slot.sold = true;
    if slot.price > m.g {
        return Err("Not enough g".to_string());
    }
    m.g -= slot.price;
    let node_id = slot.node_id;
    match slot.card_kind {
        CardKind::Unit => {
            let unit = NUnit::get(ctx, node_id)
                .to_custom_e_s_fn(|| format!("Failed to find Unit#{node_id}"))?
                .with_children(ctx)
                .with_components(ctx)
                .take();
            let house = unit
                .house
                .id
                .to_custom_e_s("NHouse parent of NUnit not found")?
                .to_node::<NHouse>(ctx)?;
            let house_name = house.house_name;
            let house = m
                .team_load(ctx)?
                .houses_load(ctx)?
                .iter()
                .find(|h| h.house_name == house_name)
                .to_custom_e_s_fn(|| format!("House {house_name} not found"))?;
            let unit = unit.clone(ctx, pid, &mut default());
            unit.id.add_parent(ctx, house.id)?;
            unit.id.add_parent(ctx, m.id)?;
        }
        CardKind::House => {
            let house = NHouse::get(ctx, node_id)
                .to_custom_e_s_fn(|| format!("Failed to find House#{node_id}"))?
                .with_components(ctx)
                .take();
            if m.team_load(ctx)?
                .houses_load(ctx)?
                .iter()
                .find(|h| h.house_name == house.house_name)
                .is_some()
            {
                // increase house lvl
            } else {
                let house = house.clone(ctx, pid, &mut default());
                house.id.add_parent(ctx, m.team_load(ctx)?.id)?;
            }
        }
    }
    m.save(ctx);
    Ok(())
}
