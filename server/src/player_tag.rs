use strum_macros::{Display, EnumString};

use super::*;

#[spacetimedb::table(public, name = player_tag)]
pub struct TPlayerTag {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub tag: String,
}

#[derive(Display, SpacetimeType, EnumString)]
pub enum PlayerTag {
    Admin,
    Supporter,
    Champion,
}

impl PlayerTag {
    fn add(self, ctx: &ReducerContext, owner: u64) {
        let tag_str = self.to_string();
        if ctx
            .db
            .player_tag()
            .owner()
            .filter(owner)
            .filter(|d| d.tag == tag_str)
            .count()
            > 0
        {
            return;
        }
        ctx.db.player_tag().insert(TPlayerTag {
            id: next_id(ctx),
            owner,
            tag: tag_str,
        });
    }
}

#[spacetimedb::reducer]
fn admin_give_tag(ctx: &ReducerContext, owner: u64, tag: String) -> Result<(), String> {
    ctx.is_admin()?;
    let tag = PlayerTag::from_str(&tag).map_err(|e| e.to_string())?;
    tag.add(ctx, owner);
    Ok(())
}
