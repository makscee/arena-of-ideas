use ::rhai::Dynamic;

use super::*;

#[derive(Clone, Debug)]
pub struct RhaiContext {
    ctx_ptr: *const u8,
}

impl RhaiContext {
    pub fn with_context(ctx: &ClientContext) -> Self {
        Self {
            ctx_ptr: ctx as *const _ as *const u8,
        }
    }

    fn get_ctx(&self) -> &ClientContext<'_> {
        unsafe { &*(self.ctx_ptr as *const ClientContext) }
    }

    pub fn get_all_units(&self) -> Vec<u64> {
        let ctx = self.get_ctx();
        if let Ok(battle) = ctx.battle() {
            battle
                .all_fusions()
                .into_iter()
                .map(|id| id as u64)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_enemies(&self, owner_id: u64) -> Vec<u64> {
        let ctx = self.get_ctx();
        if let Ok(battle) = ctx.battle() {
            if let Ok(enemies) = battle.all_enemies(owner_id as u64) {
                enemies.iter().map(|&id| id as u64).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    pub fn get_allies(&self, owner_id: u64) -> Vec<u64> {
        let ctx = self.get_ctx();
        if let Ok(battle) = ctx.battle() {
            if let Ok(allies) = battle.all_allies(owner_id as u64) {
                allies.iter().map(|&id| id as u64).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    pub fn get_adjacent_left(&self, owner_id: u64) -> Option<u64> {
        let ctx = self.get_ctx();
        if let Ok(battle) = ctx.battle() {
            battle.offset_unit(owner_id as u64, -1).map(|id| id as u64)
        } else {
            None
        }
    }

    pub fn get_adjacent_right(&self, owner_id: u64) -> Option<u64> {
        let ctx = self.get_ctx();
        if let Ok(battle) = ctx.battle() {
            battle.offset_unit(owner_id as u64, 1).map(|id| id as u64)
        } else {
            None
        }
    }

    pub fn get_adjacent_allies(&self, owner_id: u64) -> Vec<u64> {
        let ctx = self.get_ctx();
        if let Ok(battle) = ctx.battle() {
            if let Ok(allies) = battle.all_allies(owner_id as u64) {
                if let Some(pos) = allies.iter().position(|id| *id == owner_id as u64) {
                    let mut result = Vec::new();
                    if pos > 0 {
                        result.push(allies[pos - 1] as u64);
                    }
                    if pos + 1 < allies.len() {
                        result.push(allies[pos + 1] as u64);
                    }
                    return result;
                }
            }
        }
        Vec::new()
    }
}

unsafe impl Send for RhaiContext {}
unsafe impl Sync for RhaiContext {}

pub fn register_context_type(engine: &mut ::rhai::Engine) {
    engine
        .register_type_with_name::<RhaiContext>("Ctx")
        .register_fn(
            "get_all_units".register_completer(),
            |ctx: &mut RhaiContext| ctx.get_all_units(),
        )
        .register_fn(
            "get_enemies".register_completer(),
            |ctx: &mut RhaiContext, owner_id: u64| ctx.get_enemies(owner_id),
        )
        .register_fn(
            "get_allies".register_completer(),
            |ctx: &mut RhaiContext, owner_id: u64| ctx.get_allies(owner_id),
        )
        .register_fn(
            "get_adjacent_left".register_completer(),
            |ctx: &mut RhaiContext, owner_id: u64| {
                ctx.get_adjacent_left(owner_id).unwrap_or_default()
            },
        )
        .register_fn(
            "get_adjacent_right".register_completer(),
            |ctx: &mut RhaiContext, owner_id: u64| {
                ctx.get_adjacent_right(owner_id).unwrap_or_default()
            },
        )
        .register_fn(
            "get_adjacent_allies".register_completer(),
            |ctx: &mut RhaiContext, owner_id: u64| ctx.get_adjacent_allies(owner_id),
        )
        .register_fn(
            "load_unit".register_completer(),
            |ctx: &mut RhaiContext, unit_id: u64| {
                ctx.get_ctx()
                    .exec_ref(|ctx| Ok(ctx.load::<NUnit>(unit_id)?.load_all(ctx)?.take()))
                    .dynamic_result()
            },
        )
        .register_fn(
            "load_status".register_completer(),
            |ctx: &mut RhaiContext, status_id: u64| {
                ctx.get_ctx()
                    .exec_ref(|ctx| Ok(ctx.load::<NStatusMagic>(status_id)?.load_all(ctx)?.take()))
                    .dynamic_result()
            },
        )
        .register_fn(
            "load_ability".register_completer(),
            |ctx: &mut RhaiContext, ability_id: u64| {
                ctx.get_ctx()
                    .exec_ref(|ctx| {
                        Ok(ctx.load::<NAbilityMagic>(ability_id)?.load_all(ctx)?.take())
                    })
                    .dynamic_result()
            },
        )
        .register_fn(
            "load_house".register_completer(),
            |ctx: &mut RhaiContext, house_id: u64| {
                ctx.get_ctx()
                    .exec_ref(|ctx| Ok(ctx.load::<NHouse>(house_id)?.load_all(ctx)?.take()))
                    .dynamic_result()
            },
        )
        .register_fn("owner".register_completer(), |ctx: &mut RhaiContext| {
            ctx.get_ctx().owner().unwrap_or_default()
        })
        .register_fn("target".register_completer(), |ctx: &mut RhaiContext| {
            ctx.get_ctx().target().unwrap_or_default()
        })
        .register_fn(
            "get_position".register_completer(),
            |ctx: &mut RhaiContext, id: u64| {
                let ctx = ctx.get_ctx();
                match ctx.get_var_inherited(id, VarName::position).get_vec2() {
                    Ok(v) => Dynamic::from_array(v.to_array().to_dynamic_vec()),
                    Err(_) => Dynamic::UNIT,
                }
            },
        );
}
