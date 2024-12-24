mod anim;
pub mod assets;
mod context;
mod expression;
mod node_frame;
mod node_state;
mod nodes;
mod painter;
mod show;

pub use anim::*;
pub use context::*;
pub use expression::*;
pub use node_frame::*;
pub use node_state::*;
pub use nodes::*;
pub use painter::*;
pub use schema::*;
pub use show::*;

use bevy::color::Color;
use bevy::math::{vec2, Vec2};
use bevy::prelude::Mut;
use bevy::{
    ecs::system::SystemParam,
    log::*,
    prelude::{
        App, BuildChildren, Children, Commands, Component, Entity, Parent, Query, TransformBundle,
        VisibilityBundle, World,
    },
    utils::hashbrown::HashMap,
};
use bevy_egui::egui::{
    self, epaint::TextShape, Align, CollapsingHeader, Color32, Frame, Layout, Margin, Rect,
    Rounding, Shadow, Stroke, Ui,
};
use egui::{
    emath::{Rot2, TSTransform},
    epaint::{self, TessellationOptions},
    Checkbox, DragValue, Mesh, Shape, Widget,
};
use epaint::{CircleShape, RectShape, Tessellator};
use include_dir::Dir;
use itertools::Itertools;
use macro_client::*;

use parking_lot::{const_mutex, Mutex};
use std::mem;
use strum_macros::{Display, EnumIter};
use ui::*;
use utils::*;
use utils_client::{get_parent, *};

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
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        self.states
            .get(entity)
            .ok()
            .and_then(|(_, _, _, c)| c.map(|c| c.to_vec()))
            .unwrap_or_default()
    }
}
