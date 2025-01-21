use super::*;

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
