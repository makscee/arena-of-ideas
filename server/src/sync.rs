use crate::incubator::{incubator_links, incubator_votes};

use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    all: Vec<String>,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);
    for n in ctx.db.nodes_world().iter() {
        ctx.db.nodes_world().delete(n);
    }
    let all = All::from_strings(0, &all).to_e_s("Failed to parse All structure")?;
    all.save(ctx);
    Ok(())
}

#[reducer]
fn incubator_update_core(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    let mut all = All::load(ctx);
    let incubator = all.incubator_load(ctx)?;
    let units = incubator.collect_children::<Unit>(ctx);
    let mut houses: HashMap<u64, House> = HashMap::from_iter(
        incubator
            .collect_children::<House>(ctx)
            .into_iter()
            .map(|n| (n.id, n)),
    );
    for mut unit in units {
        unit.description = unit.top_link::<UnitDescription>(ctx).map(|mut d| {
            d.stats = d.top_link::<UnitStats>(ctx);
            d
        });
        if let Some(house_link) = unit.top_link_id::<House>(ctx) {
            houses.get_mut(&house_link).unwrap().units.push(unit);
        }
    }
    for house in houses.into_values() {
        house.clone(ctx, 0);
    }
    Ok(())
}
