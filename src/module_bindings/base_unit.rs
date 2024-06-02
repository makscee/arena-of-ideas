// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#[allow(unused)]
use spacetimedb_sdk::{
    anyhow::{anyhow, Result},
    identity::Identity,
    reducer::{Reducer, ReducerCallbackId, Status},
    sats::{de::Deserialize, ser::Serialize},
    spacetimedb_lib,
    table::{TableIter, TableType, TableWithPrimaryKey},
    Address,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BaseUnit {
    pub name: String,
    pub hp: i32,
    pub pwr: i32,
    pub house: String,
    pub rarity: i8,
    pub repr: u64,
    pub triggers: Vec<String>,
    pub targets: Vec<String>,
    pub effects: Vec<String>,
}

impl TableType for BaseUnit {
    const TABLE_NAME: &'static str = "BaseUnit";
    type ReducerEvent = super::ReducerEvent;
}

impl TableWithPrimaryKey for BaseUnit {
    type PrimaryKey = String;
    fn primary_key(&self) -> &Self::PrimaryKey {
        &self.name
    }
}

impl BaseUnit {
    #[allow(unused)]
    pub fn filter_by_name(name: String) -> Option<Self> {
        Self::find(|row| row.name == name)
    }
    #[allow(unused)]
    pub fn filter_by_hp(hp: i32) -> TableIter<Self> {
        Self::filter(|row| row.hp == hp)
    }
    #[allow(unused)]
    pub fn filter_by_pwr(pwr: i32) -> TableIter<Self> {
        Self::filter(|row| row.pwr == pwr)
    }
    #[allow(unused)]
    pub fn filter_by_house(house: String) -> TableIter<Self> {
        Self::filter(|row| row.house == house)
    }
    #[allow(unused)]
    pub fn filter_by_rarity(rarity: i8) -> TableIter<Self> {
        Self::filter(|row| row.rarity == rarity)
    }
    #[allow(unused)]
    pub fn filter_by_repr(repr: u64) -> TableIter<Self> {
        Self::filter(|row| row.repr == repr)
    }
    #[allow(unused)]
    pub fn filter_by_triggers(triggers: Vec<String>) -> TableIter<Self> {
        Self::filter(|row| row.triggers == triggers)
    }
    #[allow(unused)]
    pub fn filter_by_targets(targets: Vec<String>) -> TableIter<Self> {
        Self::filter(|row| row.targets == targets)
    }
    #[allow(unused)]
    pub fn filter_by_effects(effects: Vec<String>) -> TableIter<Self> {
        Self::filter(|row| row.effects == effects)
    }
}