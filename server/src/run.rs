use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
struct Run {
    #[primarykey]
    id: u64,
    #[unique]
    user_id: u64,

    team: Vec<TeamSlot>,
    shop: Vec<ShopSlot>,
    fusion_options: Vec<FusedUnit>,

    last_updated: Timestamp,
}

#[derive(SpacetimeType, Clone, Default)]
struct ShopSlot {
    unit: u64,
    price: i32,
    open: bool,
    freeze: bool,
    discount: bool,
    available: bool,
}

#[derive(SpacetimeType, Default, Clone)]
struct TeamSlot {
    unit: Option<FusedUnit>,
}

#[derive(SpacetimeType)]
pub enum FusionType {
    Trigger,
    Target,
    Effect,
}

// reducers list:
// buy unit_id
// sell unit_id
// stack unit_a, unit_b
// fuse unit_a, unit_b
// reorder

#[spacetimedb(reducer)]
fn run_start(ctx: ReducerContext) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    Run::delete_by_user_id(&user.id);
    Run::insert(Run::new(user.id))?;
    Ok(())
}

#[spacetimedb(reducer)]
fn fuse_start(ctx: ReducerContext, target: u8, source: u8) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    let mut run = Run::current(&user.id)?;
    let source_unit = run.get_team_mut(source)?.clone();
    if source_unit.bases.len() != 1 {
        return Err("Source can only be non-fused unit".to_owned());
    }
    let target_unit = run.get_team_mut(target)?.clone();

    let mut target_trigger = target_unit.clone();
    target_trigger
        .triggers
        .extend(source_unit.triggers.clone().into_iter());
    run.fusion_options.push(target_trigger);

    let mut target_target = target_unit.clone();
    target_target
        .targets
        .extend(source_unit.targets.clone().into_iter());
    run.fusion_options.push(target_target);

    let mut target_effect = target_unit.clone();
    target_effect
        .effects
        .extend(source_unit.effects.clone().into_iter());
    run.fusion_options.push(target_effect);

    run.save();
    Ok(())
}

impl Run {
    fn new(user_id: u64) -> Self {
        let gs = GlobalSettings::get();
        Self {
            id: GlobalData::next_id(),
            user_id,
            team: vec![TeamSlot::default(); gs.team_slots as usize],
            shop: vec![ShopSlot::default(); gs.shop_slots_max as usize],
            fusion_options: Vec::default(),
            last_updated: Timestamp::now(),
        }
    }

    fn current(user_id: &u64) -> Result<Self, String> {
        Run::filter_by_user_id(user_id).context_str("No arena run in progress")
    }

    fn get_team_mut(&mut self, slot: u8) -> Result<&mut FusedUnit, String> {
        self.team
            .get_mut(slot as usize)
            .and_then(|u| u.unit.as_mut())
            .context_str("Unit not found")
    }

    fn save(mut self) {
        self.last_updated = Timestamp::now();
        Self::update_by_user_id(&self.user_id.clone(), self);
    }
}
