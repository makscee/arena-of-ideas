use bevy::color::LinearRgba;
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
