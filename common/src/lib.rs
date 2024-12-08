use bevy::ecs::system::SystemParam;
use bevy::prelude::Query;
use bevy::{
    app::App,
    ecs::component::*,
    prelude::{
        debug, error, info, BuildChildren, Commands, Entity, Parent, Reflect, TransformBundle,
        VisibilityBundle, World,
    },
    utils::hashbrown::HashMap,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

pub mod assets;
pub mod effect;
pub mod event;
pub mod expression;
pub mod nodes;
pub mod trigger;

pub use assets::*;
pub use effect::*;
pub use event::*;
pub use expression::*;
pub use nodes::*;
pub use trigger::*;
use utils::*;
#[macro_use]
extern crate extra_macros;

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}

#[derive(SystemParam, Debug)]
pub struct StateQuery<'w, 's> {
    states: Query<'w, 's, (Entity, &'static NodeState, Option<&'static Parent>)>,
}

impl<'w, 's> StateQuery<'w, 's> {
    pub fn get_state(&self, entity: Entity) -> Option<&NodeState> {
        self.states.get(entity).map(|(_, s, _)| s).ok()
    }
    pub fn get_parent(&self, entity: Entity) -> Option<&Parent> {
        self.states.get(entity).ok().and_then(|(_, _, p)| p)
    }
}
