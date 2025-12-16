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

    unsafe fn get_ctx(&self) -> &ClientContext<'_> {
        &*(self.ctx_ptr as *const ClientContext)
    }

    pub fn get_all_units(&self) -> Vec<u64> {
        unsafe {
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
    }

    pub fn get_enemies(&self, owner_id: u64) -> Vec<u64> {
        unsafe {
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
    }

    pub fn get_allies(&self, owner_id: u64) -> Vec<u64> {
        unsafe {
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
    }

    pub fn get_adjacent_left(&self, owner_id: u64) -> Option<u64> {
        unsafe {
            let ctx = self.get_ctx();
            if let Ok(battle) = ctx.battle() {
                battle.offset_unit(owner_id as u64, -1).map(|id| id as u64)
            } else {
                None
            }
        }
    }

    pub fn get_adjacent_right(&self, owner_id: u64) -> Option<u64> {
        unsafe {
            let ctx = self.get_ctx();
            if let Ok(battle) = ctx.battle() {
                battle.offset_unit(owner_id as u64, 1).map(|id| id as u64)
            } else {
                None
            }
        }
    }

    pub fn get_adjacent_allies(&self, owner_id: u64) -> Vec<u64> {
        unsafe {
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

    pub fn load_unit(&self, unit_id: u64) -> Option<NUnit> {
        unsafe {
            let ctx = self.get_ctx();
            ctx.load_ref::<NUnit>(unit_id as u64)
                .ok()
                .map(|u| u.clone())
        }
    }

    pub fn load_status(&self, status_id: u64) -> Option<NStatusMagic> {
        unsafe {
            let ctx = self.get_ctx();
            ctx.load_ref::<NStatusMagic>(status_id as u64)
                .ok()
                .map(|s| s.clone())
        }
    }

    pub fn load_ability(&self, ability_id: u64) -> Option<NAbilityMagic> {
        unsafe {
            let ctx = self.get_ctx();
            ctx.load_ref::<NAbilityMagic>(ability_id as u64)
                .ok()
                .map(|a| a.clone())
        }
    }

    pub fn load_house(&self, house_id: u64) -> Option<NHouse> {
        unsafe {
            let ctx = self.get_ctx();
            ctx.load_ref::<NHouse>(house_id as u64)
                .ok()
                .map(|h| h.clone())
        }
    }
}

unsafe impl Send for RhaiContext {}
unsafe impl Sync for RhaiContext {}

pub fn register_context_type(engine: &mut ::rhai::Engine) {
    engine
        .register_type_with_name::<RhaiContext>("Ctx")
        .register_fn("get_all_units", |ctx: &mut RhaiContext| ctx.get_all_units())
        .register_fn("get_enemies", |ctx: &mut RhaiContext, owner_id: u64| {
            ctx.get_enemies(owner_id)
        })
        .register_fn("get_allies", |ctx: &mut RhaiContext, owner_id: u64| {
            ctx.get_allies(owner_id)
        })
        .register_fn(
            "get_adjacent_left",
            |ctx: &mut RhaiContext, owner_id: u64| ctx.get_adjacent_left(owner_id),
        )
        .register_fn(
            "get_adjacent_right",
            |ctx: &mut RhaiContext, owner_id: u64| ctx.get_adjacent_right(owner_id),
        )
        .register_fn(
            "get_adjacent_allies",
            |ctx: &mut RhaiContext, owner_id: u64| ctx.get_adjacent_allies(owner_id),
        )
        .register_fn("load_unit", |ctx: &mut RhaiContext, unit_id: u64| {
            ctx.load_unit(unit_id)
        })
        .register_fn("load_status", |ctx: &mut RhaiContext, status_id: u64| {
            ctx.load_status(status_id)
        })
        .register_fn("load_ability", |ctx: &mut RhaiContext, ability_id: u64| {
            ctx.load_ability(ability_id)
        })
        .register_fn("load_house", |ctx: &mut RhaiContext, house_id: u64| {
            ctx.load_house(house_id)
        });
}
