use bevy::color::LinearRgba;
use bevy::ecs::system::SystemParam;
use bevy::prelude::Query;
use bevy::{
    app::App,
    color::Color,
    ecs::component::*,
    math::Vec2,
    prelude::{
        debug, error, info, BuildChildren, Commands, Entity, Parent, Reflect, TransformBundle,
        VisibilityBundle, World,
    },
    utils::hashbrown::HashMap,
};
use bevy::{color::Srgba, math::vec2};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use thiserror::Error;

pub mod effect;
pub mod error;
pub mod event;
pub mod expression;
pub mod nodes;
pub mod trigger;
pub mod var_name;
pub mod var_value;

pub use effect::*;
pub use error::*;
pub use event::*;
pub use expression::*;
pub use nodes::*;
pub use trigger::*;
pub use var_name::*;
pub use var_value::*;
#[macro_use]
extern crate extra_macros;

pub const BEVY_MISSING_COLOR: LinearRgba = LinearRgba::new(1.0, 0.0, 1.0, 1.0);

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}

#[derive(SystemParam)]
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
