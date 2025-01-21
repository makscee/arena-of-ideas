use super::*;

fn get_match(ctx: &ReducerContext) -> Result<Match, String> {
    let id = NodeDomain::Match
        .filter_by_kind(ctx, NodeKind::Match)
        .get(0)
        .to_e_s("No matches found")?
        .id;
    Match::from_table(ctx, NodeDomain::Match, id).to_e_s("Match not found")
}

#[reducer]
fn match_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = get_match(ctx)?;
    let slot = slot as usize;
    let sc = &mut m.shop_case[slot as usize];
    let unit =
        Unit::from_table(ctx, NodeDomain::Alpha, sc.unit_id).to_e_s("Failed to find Alpha unit")?;
    sc.sold = true;
    m.g -= sc.price;
    m.team.as_mut().to_e_s("Team not set")?.units.push(unit);
    m.to_table(ctx, NodeDomain::Match, 0);
    Ok(())
}

#[reducer]
fn match_sell(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let mut m = get_match(ctx)?;
    let slot = slot as usize;
    let team = m.team.as_mut().to_e_s("Team not set")?;
    if slot >= team.units.len() {
        return Err("Slot index outside of team bounds".into());
    }
    team.units.remove(slot);
    m.to_table(ctx, NodeDomain::Match, 0);
    Ok(())
}

#[reducer]
fn match_insert(ctx: &ReducerContext) -> Result<(), String> {
    let d = Match {
        g: 13,
        shop_case: [
            ShopCaseUnit::default(),
            ShopCaseUnit::default(),
            ShopCaseUnit::default(),
        ]
        .into(),
        team: Some(Team {
            name: "Test Team".into(),
            units: [Unit {
                name: "Test Unit".into(),
                stats: Some(UnitStats {
                    pwr: 1,
                    hp: 3,
                    ..default()
                }),
                ..default()
            }]
            .into(),
            id: None,
        }),
        id: None,
    };
    d.to_table(ctx, NodeDomain::Match, 0);
    Ok(())
}

#[reducer]
fn match_get(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let d = Match::from_table(ctx, NodeDomain::Match, id);
    log::info!("{d:?}");
    Ok(())
}
