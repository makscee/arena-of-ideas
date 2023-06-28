use geng::prelude::itertools::Itertools;
use strum_macros::{Display, EnumString};

use super::*;

#[derive(
    Clone,
    Copy,
    Deserialize,
    Serialize,
    Debug,
    Eq,
    PartialEq,
    Hash,
    enum_iterator::Sequence,
    EnumString,
    Display,
)]
pub enum Faction {
    Light,
    Dark,
    Team,
    Shop,
}

impl From<f32> for Faction {
    fn from(value: f32) -> Self {
        *Faction::all_iter()
            .collect_vec()
            .get(value as usize)
            .unwrap()
    }
}

impl Into<f32> for Faction {
    fn into(self) -> f32 {
        self as i32 as f32
    }
}

impl Faction {
    pub fn color(&self, options: &Options) -> Rgba<f32> {
        match self {
            Faction::Light => options.colors.dark,
            Faction::Dark => options.colors.light,
            Faction::Team => options.colors.player,
            Faction::Shop => options.colors.shop,
        }
    }
    pub fn opposite(&self) -> Faction {
        match self {
            Faction::Light => Faction::Dark,
            Faction::Dark => Faction::Light,
            Faction::Team => Faction::Shop,
            Faction::Shop => Faction::Team,
        }
    }
    pub fn all_iter() -> impl Iterator<Item = Faction> {
        enum_iterator::all::<Faction>()
    }
    pub fn all() -> HashSet<Faction> {
        HashSet::from_iter(Self::all_iter())
    }
    pub fn battle() -> HashSet<Faction> {
        hashset! {Faction::Light, Faction::Dark}
    }
    pub fn shop() -> HashSet<Faction> {
        hashset! {Faction::Team, Faction::Shop}
    }
}
