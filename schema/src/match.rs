use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchState {
    Shop,
    RegularBattle,
    BossBattle,
    ChampionShop,
    ChampionBattle,
}

impl Default for MatchState {
    fn default() -> Self {
        MatchState::Shop
    }
}

impl MatchState {
    pub fn is_battle(self) -> bool {
        matches!(
            self,
            MatchState::RegularBattle | MatchState::BossBattle | MatchState::ChampionBattle
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
pub struct ShopOffer {
    pub buy_limit: Option<i32>,
    pub case: Vec<ShopSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
pub struct ShopSlot {
    pub card_kind: CardKind,
    pub node_id: u64,
    pub sold: bool,
    pub price: i32,
    pub buy_text: Option<String>,
}
