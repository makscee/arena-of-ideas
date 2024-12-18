pub mod assets;
mod context;
mod expression;
mod node_frame;
mod nodes;
mod show;

pub use context::*;
pub use expression::*;
pub use node_frame::*;
pub use nodes::*;
pub use schema::*;
pub use show::*;

use bevy::math::{vec2, Vec2};
use bevy::{
    ecs::system::SystemParam,
    prelude::{Children, Entity, Parent, Query},
};
use bevy_egui::egui::Color32;
use utils_client::game_timer::gt;

use utils::default;

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
