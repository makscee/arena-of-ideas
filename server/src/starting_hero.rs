use super::*;

#[spacetimedb(table)]
pub struct TStartingHero {
    #[primarykey]
    owner: u64,
    item_id: u64,
}

impl TStartingHero {
    pub fn get(owner: u64) -> Result<FusedUnit, String> {
        let d = Self::filter_by_owner(&owner).context_str("No starting hero set")?;
        let item = TItem::filter_by_id(&d.item_id).context_str("Item not found")?;
        if item.owner != owner {
            return Err(format!("Item not owned by #{owner}"));
        }
        match item.item {
            Item::Hero(unit) => Ok(unit),
            _ => {
                return Err(format!("Wrong item type for #{}", item.id));
            }
        }
    }
}

#[spacetimedb(reducer)]
fn set_starting_hero(ctx: ReducerContext, id: Option<u64>) -> Result<(), String> {
    let user = ctx.user()?;
    TStartingHero::delete_by_owner(&user.id);
    if let Some(id) = id {
        if let Some(item) = TItem::filter_by_id(&id) {
            if item.owner != user.id {
                return Err("Item not owned by user".into());
            }
            TStartingHero::insert(TStartingHero {
                owner: user.id,
                item_id: id,
            })?;
        } else {
            return Err("Item not found".into());
        }
    }
    Ok(())
}
