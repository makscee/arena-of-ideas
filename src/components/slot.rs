use super::*;

pub struct SlotComponent {
    pub slot: usize,
    pub faction: Faction,
}

impl SlotComponent {
    pub fn new(slot: usize, faction: Faction) -> Self {
        Self { slot, faction }
    }
}
