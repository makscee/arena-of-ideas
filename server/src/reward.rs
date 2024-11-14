use player_tag::player_tag;

use super::*;

#[spacetimedb::table(public, name = reward)]
pub struct TReward {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub source: String,
    pub bundle: ItemBundle,
    pub force_open: bool,
    pub ts: Timestamp,
}

impl Default for TReward {
    fn default() -> Self {
        Self {
            id: default(),
            owner: default(),
            source: default(),
            bundle: default(),
            force_open: false,
            ts: Timestamp::now(),
        }
    }
}

impl TReward {
    fn new(source: String, bundle: ItemBundle) -> Self {
        Self {
            source,
            bundle,
            ..default()
        }
    }
    pub fn force(mut self) -> Self {
        self.force_open = true;
        self
    }
    fn add(mut self, ctx: &ReducerContext, owner: u64) {
        self.owner = owner;
        self.id = next_id(ctx);
        ctx.db.reward().insert(self);
    }
    pub fn daily(ctx: &ReducerContext) {
        for player in ctx
            .db
            .player_tag()
            .iter()
            .filter(|t| PlayerTag::from_str(&t.tag).unwrap().is_supporter())
            .map(|t| t.owner)
        {
            Self::new(
                "Supporter Reward".into(),
                TLootboxItem::new(ctx, 0, LootboxKind::Regular).into(),
            )
            .add(ctx, player);
        }
    }
    pub fn lootbox_open(ctx: &ReducerContext, owner: u64, bundle: ItemBundle) {
        Self::new("Lootbox".into(), bundle).force().add(ctx, owner);
    }
}

#[spacetimedb::reducer]
fn reward_claim(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let reward: TReward = ctx
        .db
        .reward()
        .id()
        .find(id)
        .context_str("Reward not found")?;
    player.check_owner(reward.owner)?;
    reward.bundle.take(ctx, player.id)?;
    ctx.db.reward().id().delete(id);
    Ok(())
}
