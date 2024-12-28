mod anim;
pub mod assets;
mod context;
mod data_frame;
mod expression;
mod node_frame;
mod node_state;
mod nodes;
mod painter;
mod show;
mod tween;
mod vfx;

pub use anim::*;
pub use context::*;
pub use data_frame::*;
pub use expression::*;
pub use node_frame::*;
pub use node_state::*;
pub use nodes::*;
pub use painter::*;
pub use schema::*;
pub use show::*;
pub use tween::*;
pub use vfx::*;

use bevy::{
    color::{Color, Mix},
    ecs::system::SystemParam,
    log::*,
    math::{vec2, Vec2},
    prelude::{
        App, BuildChildren, Children, Commands, Component, Entity, Mut, Parent, Query,
        TransformBundle, VisibilityBundle, World,
    },
    utils::hashbrown::HashMap,
};
use bevy_egui::egui::{
    self, epaint::TextShape, Align, CollapsingHeader, Color32, Frame, Layout, Margin, Rect,
    Rounding, Shadow, Stroke, Ui,
};
use egui::{
    emath::{self, Rot2, TSTransform},
    epaint::{self, TessellationOptions},
    remap, Checkbox, DragValue, Mesh, Response, Sense, Widget,
};
use epaint::{CircleShape, RectShape, Tessellator};
use include_dir::Dir;
use itertools::Itertools;
use macro_client::*;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    f32::consts::TAU,
    ops::{Deref, DerefMut},
};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};
use utils_client::{ToC32, ToColor};

use parking_lot::{const_mutex, Mutex};
use std::mem;
use strum_macros::Display;
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
