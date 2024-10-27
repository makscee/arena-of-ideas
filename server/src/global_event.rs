use auction::TAuction;

use super::*;

#[spacetimedb(table(public))]
struct TGlobalEvent {
    #[primarykey]
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
    QuestAccepted(TQuest),
    QuestComplete(TQuest),
    Fuse(FusedUnit),
}

impl GlobalEvent {
    pub fn post(self, owner: u64) {
        TGlobalEvent::insert(TGlobalEvent {
            id: next_id(),
            owner,
            event: self,
            ts: Timestamp::now(),
        })
        .unwrap();
    }
}
