mod anim;
mod assets;
mod battle;
mod battle_action;
mod context;
mod data_frame;
mod effect;
mod expression;
mod node_frame;
mod node_state;
mod node_state_plugin;
mod nodes;
mod painter;
mod show;
mod tween;

use super::*;
pub use anim::*;
pub use assets::*;
pub use battle::*;
pub use battle_action::*;
pub use context::*;
pub use data_frame::*;
pub use effect::*;
pub use expression::*;
pub use node_frame::*;
pub use node_state::*;
pub use node_state_plugin::*;
pub use nodes::*;
pub use painter::*;
pub use schema::*;
pub use show::*;
pub use tween::*;

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

pub trait EntityConverter {
    fn to_e(&self) -> Entity;
}

impl EntityConverter for u64 {
    fn to_e(&self) -> Entity {
        Entity::from_bits(*self)
    }
}
