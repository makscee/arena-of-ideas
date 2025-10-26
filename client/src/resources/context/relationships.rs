use crate::prelude::*;
use bevy::prelude::BuildChildrenTransformExt;
// ChildOf and Children are imported from bevy::prelude in the prelude

// Node kind marker component
#[derive(BevyComponent, Clone, Debug)]
pub struct NodeKindMarker(pub NodeKind);

// Node ID component
#[derive(BevyComponent, Clone, Debug)]
pub struct NodeId(pub u64);

// Node owner component
#[derive(BevyComponent, Clone, Debug)]
pub struct NodeOwner(pub u64);

// Custom relationship marker for reference links (non-hierarchical)
#[derive(BevyComponent, Clone, Debug)]
pub struct ReferenceTo(pub Entity);

// Custom relationship marker for component grouping
#[derive(BevyComponent, Clone, Debug)]
pub struct ComponentGroupOf(pub Entity);

// Helper functions for working with hierarchical relationships (owned links)
pub fn add_owned_relationship(world: &mut World, parent: Entity, child: Entity) {
    world.entity_mut(child).set_parent_in_place(parent);
}

pub fn remove_owned_relationship(world: &mut World, child: Entity) {
    world.entity_mut(child).remove_parent_in_place();
}

pub fn get_owned_children(world: &World, parent: Entity) -> Vec<Entity> {
    world
        .get::<Children>(parent)
        .map(|children| children.iter().copied().collect())
        .unwrap_or_default()
}

pub fn get_owned_parent(world: &World, child: Entity) -> Option<Entity> {
    world.get::<ChildOf>(child).map(|child_of| child_of.0)
}

// Helper functions for reference relationships (non-hierarchical)
pub fn add_reference_relationship(world: &mut World, parent: Entity, child: Entity) {
    world.entity_mut(child).insert(ReferenceTo(parent));
}

pub fn remove_reference_relationship(world: &mut World, child: Entity) {
    world.entity_mut(child).remove::<ReferenceTo>();
}

pub fn get_referenced_children(world: &World, parent: Entity) -> Vec<Entity> {
    let mut children = Vec::new();
    for entity in world.iter_entities() {
        if let Some(reference) = world.get::<ReferenceTo>(entity.id()) {
            if reference.0 == parent {
                children.push(entity.id());
            }
        }
    }
    children
}

pub fn get_reference_parent(world: &World, child: Entity) -> Option<Entity> {
    world.get::<ReferenceTo>(child).map(|refs| refs.0)
}

// Helper functions for component grouping
pub fn add_component_relationship(world: &mut World, parent: Entity, child: Entity) {
    world.entity_mut(child).insert(ComponentGroupOf(parent));
}

pub fn remove_component_relationship(world: &mut World, child: Entity) {
    world.entity_mut(child).remove::<ComponentGroupOf>();
}

pub fn get_component_siblings(world: &World, entity: Entity) -> Vec<Entity> {
    if let Some(group_of) = world.get::<ComponentGroupOf>(entity) {
        let parent = group_of.0;
        let mut siblings = Vec::new();
        for sibling_entity in world.iter_entities() {
            if let Some(sibling_group) = world.get::<ComponentGroupOf>(sibling_entity.id()) {
                if sibling_group.0 == parent && sibling_entity.id() != entity {
                    siblings.push(sibling_entity.id());
                }
            }
        }
        siblings
    } else {
        Vec::new()
    }
}

pub fn get_component_parent(world: &World, child: Entity) -> Option<Entity> {
    world.get::<ComponentGroupOf>(child).map(|comp| comp.0)
}

// Helper to check if an entity has any of our relationship components
pub fn has_relationships(world: &World, entity: Entity) -> bool {
    world.get::<ChildOf>(entity).is_some()
        || world.get::<ReferenceTo>(entity).is_some()
        || world.get::<ComponentGroupOf>(entity).is_some()
}

// Helper to remove all relationships from an entity
pub fn remove_all_relationships(world: &mut World, entity: Entity) {
    let mut entity_mut = world.entity_mut(entity);

    // Remove hierarchical relationship
    if entity_mut.contains::<ChildOf>() {
        entity_mut.remove_parent_in_place();
    }

    // Remove reference relationship
    if entity_mut.contains::<ReferenceTo>() {
        entity_mut.remove::<ReferenceTo>();
    }

    // Remove component grouping relationship
    if entity_mut.contains::<ComponentGroupOf>() {
        entity_mut.remove::<ComponentGroupOf>();
    }
}

// Query helpers for finding entities by relationship type
pub fn find_entities_with_node_kind(world: &World, kind: NodeKind) -> Vec<Entity> {
    let mut entities = Vec::new();
    for entity in world.iter_entities() {
        if let Some(marker) = world.get::<NodeKindMarker>(entity.id()) {
            if marker.0 == kind {
                entities.push(entity.id());
            }
        }
    }
    entities
}

pub fn find_entity_by_node_id(world: &World, node_id: u64) -> Option<Entity> {
    for entity in world.iter_entities() {
        if let Some(id) = world.get::<NodeId>(entity.id()) {
            if id.0 == node_id {
                return Some(entity.id());
            }
        }
    }
    None
}

pub fn find_entities_by_owner(world: &World, owner_id: u64) -> Vec<Entity> {
    let mut entities = Vec::new();
    for entity in world.iter_entities() {
        if let Some(owner) = world.get::<NodeOwner>(entity.id()) {
            if owner.0 == owner_id {
                entities.push(entity.id());
            }
        }
    }
    entities
}
