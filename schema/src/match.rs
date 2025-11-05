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

impl ShopOffer {
    pub fn get_slot_mut<'a>(&'a mut self, i: u8) -> Result<&'a mut ShopSlot, String> {
        self.case
            .get_mut(i as usize)
            .to_custom_e_s_fn(|| format!("Failed to get shop slot {i}"))
    }
}

impl ShopSlot {
    pub fn units_from_ids(ids: Vec<u64>, price: i32) -> Vec<Self> {
        ids.into_iter()
            .map(|id| Self {
                card_kind: CardKind::Unit,
                node_id: id,
                sold: false,
                price,
                buy_text: None,
            })
            .collect()
    }
}
