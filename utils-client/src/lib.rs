pub mod game_timer;
pub mod operations;

use bevy::prelude::*;

pub fn get_children(entity: Entity, world: &World) -> Vec<Entity> {
    world
        .get::<Children>(entity)
        .map(|c| c.to_vec())
        .unwrap_or_default()
}
pub fn get_children_recursive(entity: Entity, world: &World) -> Vec<Entity> {
    let mut children = get_children(entity, world);
    let mut i = 0;
    while i < children.len() {
        children.extend(get_children(children[i], world));
        i += 1;
    }
    children
}
pub fn get_parent(entity: Entity, world: &World) -> Option<Entity> {
    world.get::<Parent>(entity).map(|p| p.get())
}
