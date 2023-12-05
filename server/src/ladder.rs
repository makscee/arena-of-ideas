use super::*;

#[spacetimedb(table)]
pub struct Ladder {
    #[primarykey]
    id: u64,
    owner: Identity,
    levels: Vec<String>,
}

#[spacetimedb(reducer)]
pub fn add_user_ladder(ctx: ReducerContext, levels: Vec<String>) -> Result<(), String> {
    for ladder in Ladder::filter_by_owner(&ctx.sender) {
        if ladder.levels.len() < levels.len() {
            Ladder::delete_by_id(&ladder.id);
        } else {
            return Ok(());
        }
    }
    match Ladder::insert(Ladder {
        id: 0,
        owner: ctx.sender.clone(),
        levels,
    }) {
        Ok(Ladder {
            id,
            owner: _,
            levels,
        }) => {
            info!("New ladder added {id} {levels:?}");
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
