use auction::TAuction;
use spacetimedb::Table;

use super::*;

#[spacetimedb::table(name = global_event)]
struct TGlobalEvent {
    #[primary_key]
    id: u64,
    owner: u64,
    event: GlobalEvent,
    ts: Timestamp,
}

#[derive(SpacetimeType)]
#[must_use]
pub enum GlobalEvent {
    LogIn,
    LogOut,
    Register,
    RunStart(GameMode),
    RunFinish(TArenaRun),
    BattleFinish(TBattle),
    MetaShopBuy(TMetaShop),
    GameShopBuy(String),
    GameShopSkip(String),
    GameShopSell(FusedUnit),
    AuctionBuy(TAuction),
    AuctionCancel(TAuction),
    AuctionPost(TAuction),
    ReceiveUnit(TUnitItem),
    DismantleUnit(TUnitItem),
    CraftUnit(TUnitItem),
    ReceiveUnitShard(TUnitShardItem),
    ReceiveRainbowShard(TRainbowShardItem),
    ReceiveLootbox(TLootboxItem),
    OpenLootbox(TLootboxItem),
    QuestAccepted(TQuest),
    QuestComplete(TQuest),
    Fuse(FusedUnit),
}

impl GlobalEvent {
    pub fn post(self, ctx: &ReducerContext, owner: u64) {
        ctx.db.global_event().insert(TGlobalEvent {
            id: next_id(ctx),
            owner,
            event: self,
            ts: Timestamp::now(),
        });
    }
}
