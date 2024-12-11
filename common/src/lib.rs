use ::ui::*;
use ::ui::{Show, VISIBLE_LIGHT};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Query};
use bevy::{
    app::App,
    ecs::component::*,
    prelude::{
        debug, error, info, BuildChildren, Commands, Entity, Parent, Reflect, TransformBundle,
        VisibilityBundle, World,
    },
    utils::hashbrown::HashMap,
};
use bevy_egui::egui::{self, Color32, Frame, Margin, Rounding, Stroke, Ui};
use egui::{CollapsingHeader, Shadow};
use include_dir::Dir;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

mod assets;
mod effect;
mod event;
mod expression;
mod material;
mod node_frame;
mod nodes;
mod trigger;

pub use assets::*;
pub use effect::*;
pub use event::*;
pub use expression::*;
pub use material::*;
pub use node_frame::*;
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
    states: Query<
        'w,
        's,
        (
            Entity,
            &'static NodeState,
            Option<&'static Parent>,
            Option<&'static Children>,
        ),
    >,
}

impl<'w, 's> StateQuery<'w, 's> {
    pub fn get_state(&self, entity: Entity) -> Option<&NodeState> {
        self.states.get(entity).map(|(_, s, _, _)| s).ok()
    }
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.states
            .get(entity)
            .ok()
            .and_then(|(_, _, p, _)| p.map(|p| p.get()))
    }
}
